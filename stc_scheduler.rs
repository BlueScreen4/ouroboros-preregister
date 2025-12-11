use dashmap::DashMap;
use parking_lot::{Mutex, RwLock};
use serde::{Deserialize, Serialize};
use std::fs;
use std::time::Instant;
use tracing::{info, warn};

use crate::lib::stc;
use stc::{AdminRequestPayload, OffloadRequestPayload, ServerCommand};
use stc::server_command::{CommandType as ServerCmdType, Payload as ServerPayload};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum NodeTier {
    Offline = 0,
    Tier3Mobile = 1,
    Tier2Standard = 2,
    Tier1HighPerformance = 3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HealthState {
    Healthy,
    Degraded,
    Suspect,
    Quarantined,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerInfo {
    pub id: String,
    pub name: String,
    pub domain: String,
    pub ai_models: Vec<String>,
    pub description: String,
    pub status: String,
    pub required_vram_gb: u32,
}

#[derive(Debug, Clone)]
pub struct NodeContext {
    pub node_id: String,
    pub device_model: String,
    pub cpu_cores: u32,
    pub total_ram_mb: u64,
    pub has_npu: bool,
    pub has_cuda: bool,
    pub has_rocm: bool,
    pub has_intel_arc: bool,
    pub pcie_lanes: u32,
    pub pcie_gen: u32,
    pub memory_bandwidth_gbps: f64,
    pub compute_units: u32,
    pub current_tier: NodeTier,
    pub last_seen: Instant,
    pub cpu_load: f64,
    pub gpu_load: f64,
    pub is_charging: bool,
    pub network_type: String,
    pub user_allowed: bool,

    // ==== OPI 3.0 / 네트워크 / 셀프힐링 ====
    pub net_rtt_ema_ms: f64,     // Heartbeat 기반 RTT EMA
    pub health_state: HealthState,
    pub failure_count: u32,      // 연속 실패/타임아웃 횟수
    pub is_quarantined: bool,    // 스케줄링 대상 제외 여부
}

#[derive(Debug, Clone, Copy)]
pub struct OverloadThresholds {
    pub cpu_max: f64,
    pub gpu_max: f64,
    pub vram_pressure_max: f64,
}

#[derive(Debug, Default)]
pub struct ServerStatus {
    pub cpu_load: f64,
    pub gpu_load: f64,
    pub vram_usage_ratio: f64,
}

#[derive(Debug)]
pub struct StcScheduler {
    pub master_id: String,
    pub nodes: DashMap<String, NodeContext>,
    pub thresholds: OverloadThresholds,
    pub server_status: Mutex<ServerStatus>,
    pub container_registry: RwLock<Vec<ContainerInfo>>,
}

impl StcScheduler {
    pub fn new(master_id: String, thresholds: OverloadThresholds) -> Self {
        let registry = Self::load_containers_from_file("containers.json");
        Self {
            master_id,
            nodes: DashMap::new(),
            thresholds,
            server_status: Mutex::new(ServerStatus::default()),
            container_registry: RwLock::new(registry),
        }
    }

    fn load_containers_from_file(path: &str) -> Vec<ContainerInfo> {
        fs::read_to_string(path)
            .ok()
            .and_then(|c| serde_json::from_str(&c).ok())
            .unwrap_or_default()
    }

    // ---------------- Node 등록 / 상태 ----------------

    pub fn register_node_ctx(&self, mut ctx: NodeContext) {
        // 초기 헬스/네트워크 값 세팅
        ctx.net_rtt_ema_ms = 0.0;
        ctx.health_state = HealthState::Healthy;
        ctx.failure_count = 0;
        ctx.is_quarantined = false;

        let raw_score = self.calculate_raw_opi(&ctx);
        ctx.current_tier = self.determine_tier(raw_score);

        let node_id = ctx.node_id.clone();
self.nodes.insert(node_id.clone(), ctx);

if let Some(inserted) = self.nodes.get(&node_id) {
    info!(
        "[Scheduler] Node Registered: {} (OPI: {:.1}, Tier: {:?})",
        node_id,
        raw_score,
        inserted.current_tier
    );
} else {
    info!(
        "[Scheduler] Node Registered: {} (OPI: {:.1})",
        node_id, raw_score
    );
}

    }

    fn calculate_raw_opi(&self, node: &NodeContext) -> f64 {
        let mut score = 0.0;
        score += (node.total_ram_mb as f64 / 1024.0) * 5.0;
        score += node.memory_bandwidth_gbps / 10.0;
        score += (node.pcie_lanes * node.pcie_gen) as f64 * 2.0;
        score += node.compute_units as f64 * 0.5;
        if node.has_rocm {
            score *= 1.1;
        }
        score
    }

    fn determine_tier(&self, score: f64) -> NodeTier {
        if score >= 200.0 {
            NodeTier::Tier1HighPerformance
        } else if score >= 80.0 {
            NodeTier::Tier2Standard
        } else {
            NodeTier::Tier3Mobile
        }
    }

    pub fn update_node_status(
        &self,
        id: &str,
        cpu: f64,
        gpu: f64,
        charging: bool,
        net: String,
        allowed: bool,
    ) {
        // 기존 API 유지용: RTT 없이 호출되면 RTT=0으로 처리
        self.update_node_status_with_rtt(id, cpu, gpu, charging, net, allowed, 0.0);
    }

    pub fn update_node_status_with_rtt(
        &self,
        id: &str,
        cpu: f64,
        gpu: f64,
        charging: bool,
        net: String,
        allowed: bool,
        rtt_ms: f64,
    ) {
        if let Some(mut node) = self.nodes.get_mut(id) {
            node.cpu_load = cpu;
            node.gpu_load = gpu;
            node.is_charging = charging;
            node.network_type = net;
            node.user_allowed = allowed;
            node.last_seen = Instant::now();

            // RTT EMA 업데이트 (0이면 업데이트 생략)
            let gamma = 0.2_f64;
            if rtt_ms > 0.0 {
                if node.net_rtt_ema_ms <= 0.0 {
                    node.net_rtt_ema_ms = rtt_ms;
                } else {
                    node.net_rtt_ema_ms =
                        gamma * rtt_ms + (1.0 - gamma) * node.net_rtt_ema_ms;
                }
            }

            // 헬스 상태 갱신
            self.update_health_state(&mut node);

            // Tier 재계산 (하드웨어 기반)
            let score = self.calculate_raw_opi(&node);
            let new_tier = self.determine_tier(score);
            if node.current_tier != new_tier {
                info!(
                    "[Tier Change] {}: {:?} -> {:?}",
                    node.node_id, node.current_tier, new_tier
                );
                node.current_tier = new_tier;
            }
        } else {
            warn!("[Scheduler] update_node_status_with_rtt: unknown node_id={}", id);
        }
    }

    fn update_health_state(&self, node: &mut NodeContext) {
        use HealthState::*;

        let now = Instant::now();
        let since_seen = now.duration_since(node.last_seen).as_secs_f64();

        // 하드 타임아웃 기준
        if since_seen > 30.0 {
            node.health_state = Quarantined;
            node.is_quarantined = true;
            return;
        } else if since_seen > 10.0 {
            node.health_state = Suspect;
            // 스케줄링에서는 제외하되, 일단 완전 격리는 아님
            return;
        }

        // RTT 기반 상태 (Heartbeat는 오고 있다고 가정)
        if node.net_rtt_ema_ms > 150.0 {
            node.health_state = Degraded;
            node.is_quarantined = false;
        } else {
            node.health_state = Healthy;
            node.is_quarantined = false;
        }
    }

    fn calculate_net_factor(&self, node: &NodeContext) -> f64 {
        let base_rtt_ms = 10.0_f64; // "정상" LAN 기준
        let max_penalty = 10.0_f64; // 최대 10배 페널티

        let rtt = if node.net_rtt_ema_ms <= 0.0 {
            base_rtt_ms
        } else {
            node.net_rtt_ema_ms
        };

        let raw = rtt / base_rtt_ms;
        raw.clamp(1.0, max_penalty)
    }

    fn calculate_load_factor(&self, node: &NodeContext) -> f64 {
        let load = node.gpu_load.max(node.cpu_load).clamp(0.0, 1.0);
        1.0 - load
    }

    fn calculate_effective_opi(&self, node: &NodeContext) -> f64 {
        use HealthState::*;

        if node.is_quarantined || matches!(node.health_state, Quarantined | Suspect) {
            return 0.0;
        }

        let hw = self.calculate_raw_opi(node);
        let net = self.calculate_net_factor(node);
        let load_factor = self.calculate_load_factor(node);

        (hw / net) * load_factor
    }

    pub fn update_master_status(&self, cpu: f64, gpu: f64, vram_ratio: f64) {
        let mut status = self.server_status.lock();
        status.cpu_load = cpu;
        status.gpu_load = gpu;
        status.vram_usage_ratio = vram_ratio;
    }

    // ---------------- Smart Sharding ----------------

    pub fn check_server_overload_and_shard(&self) -> Vec<(String, ServerCommand)> {
        let status = self.server_status.lock();
        let mut commands = Vec::new();

        if status.cpu_load > self.thresholds.cpu_max
            || status.vram_usage_ratio > self.thresholds.vram_pressure_max
        {
            let candidates = self.find_smart_candidates();
            for node_id in candidates.iter().take(3) {
                commands.push((node_id.clone(), self.create_shard_command()));
            }
        }

        commands
    }

    fn find_smart_candidates(&self) -> Vec<String> {
        let mut candidates: Vec<(String, f64)> = self
            .nodes
            .iter()
            .filter_map(|entry| {
                let node = entry.value();

                if !node.user_allowed || node.current_tier == NodeTier::Offline {
                    return None;
                }

                // 과부하 노드 제외
                if node.cpu_load > 0.9 || node.gpu_load > 0.9 {
                    return None;
                }

                // 헬스/격리 상태 반영
                if node.is_quarantined {
                    return None;
                }

                let eff_opi = self.calculate_effective_opi(node);
                if eff_opi <= 0.0 {
                    return None;
                }

                Some((node.node_id.clone(), eff_opi))
            })
            .collect();

        candidates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        candidates.into_iter().map(|(id, _)| id).collect()
    }

    fn create_shard_command(&self) -> ServerCommand {
        use stc::ShardPayload;

        let shard_id = uuid::Uuid::new_v4().to_string();

        ServerCommand {
            r#type: ServerCmdType::ShardTask as i32,
            task_id: shard_id.clone(),
            payload: Some(ServerPayload::Shard(ShardPayload {
                shard_id,
                shard_index: 0,
                shard_total: 1,
                data: Vec::new(),
                next_container: "Programming".into(),
                buffer_tag: "default".into(),
            })),
        }
    }

    // ---------------- Offload / Admin ----------------

    pub fn handle_offload_request(
        &self,
        client_id: &str,
        req: &OffloadRequestPayload,
    ) -> Option<ServerCommand> {
        info!(
            "[Offload] {} -> container={} task_type={} model={}",
            client_id, req.container_id, req.task_type, req.model_variant
        );

        Some(ServerCommand {
            r#type: ServerCmdType::OffloadAccepted as i32,
            task_id: format!("offload_{}", client_id),
            payload: None,
        })
    }

    pub fn handle_babel_request(
        &self,
        client_id: &str,
        _req: &stc::BabelRequestPayload,
    ) -> Option<ServerCommand> {
        info!("[Babel] Session start: client={}", client_id);

        Some(ServerCommand {
            r#type: ServerCmdType::StreamInit as i32,
            task_id: format!("babel_{}", client_id),
            payload: None,
        })
    }

    pub fn handle_assist_request(
        &self,
        _client_id: &str,
        _req: &stc::AssistRequestPayload,
    ) -> Option<ServerCommand> {
        None
    }

    pub fn handle_admin_action(&self, req: &AdminRequestPayload) -> Result<String, String> {
        info!(
            "[Admin] Action: {} target={} msg={}",
            req.action, req.target, req.message
        );
        Ok("Processed".into())
    }
}
