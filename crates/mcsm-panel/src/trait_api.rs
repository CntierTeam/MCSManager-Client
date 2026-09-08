use async_trait::async_trait;
use mcsm_protocol::auth::*;
use mcsm_protocol::environment::*;
use mcsm_protocol::error::McsmResult;
use mcsm_protocol::exchange::*;
use mcsm_protocol::files::*;
use mcsm_protocol::ids::{DaemonId, InstanceUuid};
use mcsm_protocol::instance::*;
use mcsm_protocol::java::*;
use mcsm_protocol::mod_mgr::*;
use mcsm_protocol::overview::*;
use mcsm_protocol::schedule::*;
use mcsm_protocol::service::*;
use mcsm_protocol::stream::{FilePassport, StreamPassport};
use serde_json::Value;

/// Frozen Panel client contract. Implementations live in this crate only.
#[async_trait]
pub trait PanelClient: Send + Sync {
    // --- Auth ---
    async fn login(&self, user: &str, pass: &str, code: Option<&str>) -> McsmResult<LoginOut>;
    async fn logout(&self) -> McsmResult<()>;
    async fn status(&self) -> McsmResult<AuthStatus>;
    async fn login_info(&self) -> McsmResult<LoginPageInfo>;
    async fn install(&self, req: &InstallRequest) -> McsmResult<Value>;
    async fn me(&self) -> McsmResult<UserInfo>;
    async fn update_profile(&self, req: &UpdatePasswordRequest) -> McsmResult<Value>;
    async fn set_api_key(&self, enable: bool) -> McsmResult<Value>;
    async fn bind2fa(&self) -> McsmResult<Value>;
    async fn confirm2fa(&self, code: &str) -> McsmResult<Value>;
    async fn create_user(&self, req: &CreateUserRequest) -> McsmResult<Value>;
    async fn delete_user(&self, uuid: &str) -> McsmResult<Value>;
    async fn edit_user(&self, req: &EditUserRequest) -> McsmResult<Value>;
    async fn search_users(&self, q: &UserSearchQuery) -> McsmResult<UserPage>;
    async fn user_overview(&self) -> McsmResult<Value>;
    async fn query_username(&self, username: &str) -> McsmResult<Value>;
    async fn sso_config(&self) -> McsmResult<Value>;

    // --- Overview / settings / audit ---
    async fn overview(&self) -> McsmResult<Overview>;
    async fn operation_logs(&self) -> McsmResult<Value>;
    async fn search_operation_logs(&self, q: &AuditQuery) -> McsmResult<AuditPage>;
    async fn get_setting(&self) -> McsmResult<PanelSetting>;
    async fn put_setting(&self, s: &PanelSetting) -> McsmResult<Value>;
    async fn get_layout(&self) -> McsmResult<LayoutData>;
    async fn put_layout(&self, layout: &LayoutData) -> McsmResult<Value>;
    async fn delete_layout(&self) -> McsmResult<Value>;
    async fn install_language(&self, language: &str) -> McsmResult<Value>;

    // --- Nodes ---
    async fn list_nodes(&self) -> McsmResult<Vec<NodeSummary>>;
    async fn list_remote_services(&self) -> McsmResult<Value>;
    async fn remote_services_system(&self) -> McsmResult<Value>;
    async fn instances_on_node(&self, q: &InstanceListQuery) -> McsmResult<InstancePage>;
    async fn instances_global(&self, q: &GlobalInstanceQuery) -> McsmResult<Value>;
    async fn add_node(&self, req: &AddNode) -> McsmResult<Value>;
    async fn edit_node(&self, uuid: &DaemonId, req: &EditNode) -> McsmResult<Value>;
    async fn delete_node(&self, uuid: &DaemonId) -> McsmResult<Value>;
    async fn reconnect_node(&self, uuid: &DaemonId) -> McsmResult<Value>;

    // --- Instance admin ---
    async fn instance_detail(&self, daemon: &DaemonId, uuid: &InstanceUuid)
        -> McsmResult<Instance>;
    async fn create_instance(&self, daemon: &DaemonId, req: &CreateInstance) -> McsmResult<Value>;
    async fn update_instance(
        &self,
        daemon: &DaemonId,
        uuid: &InstanceUuid,
        req: &UpdateInstance,
    ) -> McsmResult<Value>;
    async fn delete_instances(&self, daemon: &DaemonId, req: &DeleteInstances)
        -> McsmResult<Value>;
    async fn multi_open(&self, body: &Value) -> McsmResult<Value>;
    async fn multi_stop(&self, body: &Value) -> McsmResult<Value>;
    async fn multi_kill(&self, body: &Value) -> McsmResult<Value>;
    async fn multi_restart(&self, body: &Value) -> McsmResult<Value>;
    async fn quick_install_list(&self) -> McsmResult<Value>;
    async fn instance_upload_passport(
        &self,
        daemon: &DaemonId,
        upload_dir: &str,
    ) -> McsmResult<FilePassport>;

    // --- Protected instance ops ---
    async fn instance_open(&self, daemon: &DaemonId, uuid: &InstanceUuid) -> McsmResult<Value>;
    async fn instance_stop(&self, daemon: &DaemonId, uuid: &InstanceUuid) -> McsmResult<Value>;
    async fn instance_restart(&self, daemon: &DaemonId, uuid: &InstanceUuid) -> McsmResult<Value>;
    async fn instance_kill(&self, daemon: &DaemonId, uuid: &InstanceUuid) -> McsmResult<Value>;
    async fn instance_command(
        &self,
        daemon: &DaemonId,
        uuid: &InstanceUuid,
        command: &str,
    ) -> McsmResult<Value>;
    async fn stream_channel(
        &self,
        daemon: &DaemonId,
        uuid: &InstanceUuid,
    ) -> McsmResult<StreamPassport>;
    async fn outputlog(&self, daemon: &DaemonId, uuid: &InstanceUuid) -> McsmResult<String>;
    async fn process_config_list(
        &self,
        daemon: &DaemonId,
        uuid: &InstanceUuid,
    ) -> McsmResult<Value>;
    async fn process_config_get(
        &self,
        daemon: &DaemonId,
        uuid: &InstanceUuid,
        file_name: &str,
    ) -> McsmResult<Value>;
    async fn process_config_put(
        &self,
        daemon: &DaemonId,
        uuid: &InstanceUuid,
        file_name: &str,
        body: &Value,
    ) -> McsmResult<Value>;
    async fn instance_update_user(
        &self,
        daemon: &DaemonId,
        uuid: &InstanceUuid,
        body: &Value,
    ) -> McsmResult<Value>;
    async fn asynchronous(
        &self,
        daemon: &DaemonId,
        uuid: &InstanceUuid,
        task_name: &str,
        body: Option<&Value>,
    ) -> McsmResult<Value>;
    async fn query_asynchronous(
        &self,
        daemon: &DaemonId,
        uuid: &InstanceUuid,
        task_name: &str,
    ) -> McsmResult<Value>;
    async fn stop_asynchronous(
        &self,
        daemon: &DaemonId,
        uuid: &InstanceUuid,
        task_name: &str,
    ) -> McsmResult<Value>;
    async fn install_instance(
        &self,
        daemon: &DaemonId,
        uuid: &InstanceUuid,
        body: &Value,
    ) -> McsmResult<Value>;

    // --- Files ---
    async fn file_status(&self, daemon: &DaemonId, uuid: &InstanceUuid) -> McsmResult<FileStatus>;
    async fn file_list(
        &self,
        daemon: &DaemonId,
        uuid: &InstanceUuid,
        q: &FileListQuery,
    ) -> McsmResult<FileList>;
    async fn file_edit(
        &self,
        daemon: &DaemonId,
        uuid: &InstanceUuid,
        body: &FileEditBody,
    ) -> McsmResult<Value>;
    async fn file_touch(
        &self,
        daemon: &DaemonId,
        uuid: &InstanceUuid,
        target: &str,
    ) -> McsmResult<Value>;
    async fn file_mkdir(
        &self,
        daemon: &DaemonId,
        uuid: &InstanceUuid,
        target: &str,
    ) -> McsmResult<Value>;
    async fn file_copy(
        &self,
        daemon: &DaemonId,
        uuid: &InstanceUuid,
        body: &Value,
    ) -> McsmResult<Value>;
    async fn file_move(
        &self,
        daemon: &DaemonId,
        uuid: &InstanceUuid,
        body: &Value,
    ) -> McsmResult<Value>;
    async fn file_delete(
        &self,
        daemon: &DaemonId,
        uuid: &InstanceUuid,
        body: &Value,
    ) -> McsmResult<Value>;
    async fn file_chmod(
        &self,
        daemon: &DaemonId,
        uuid: &InstanceUuid,
        body: &FileChmodBody,
    ) -> McsmResult<Value>;
    async fn file_compress(
        &self,
        daemon: &DaemonId,
        uuid: &InstanceUuid,
        body: &Value,
    ) -> McsmResult<Value>;
    async fn file_download_from_url(
        &self,
        daemon: &DaemonId,
        uuid: &InstanceUuid,
        body: &DownloadFromUrlBody,
    ) -> McsmResult<Value>;
    async fn file_download_passport(
        &self,
        daemon: &DaemonId,
        uuid: &InstanceUuid,
        file_name: &str,
    ) -> McsmResult<FilePassport>;
    async fn file_upload_passport(
        &self,
        daemon: &DaemonId,
        uuid: &InstanceUuid,
        upload_dir: &str,
    ) -> McsmResult<FilePassport>;

    // --- Schedule ---
    async fn schedule_list(
        &self,
        daemon: &DaemonId,
        uuid: &InstanceUuid,
    ) -> McsmResult<Vec<ScheduleTask>>;
    async fn schedule_create(
        &self,
        daemon: &DaemonId,
        uuid: &InstanceUuid,
        body: &CreateSchedule,
    ) -> McsmResult<Value>;
    async fn schedule_delete(
        &self,
        daemon: &DaemonId,
        uuid: &InstanceUuid,
        name: &str,
    ) -> McsmResult<Value>;

    // --- Environment / Java / Mod / Exchange ---
    async fn env_images(&self, daemon: &DaemonId) -> McsmResult<Vec<DockerImage>>;
    async fn env_new_image(&self, daemon: &DaemonId, body: &NewImageBody) -> McsmResult<Value>;
    async fn env_del_image(&self, daemon: &DaemonId, image_id: &str) -> McsmResult<Value>;
    async fn env_containers(&self, daemon: &DaemonId) -> McsmResult<Value>;
    async fn env_network_modes(&self, daemon: &DaemonId) -> McsmResult<Value>;
    async fn env_progress(&self, daemon: &DaemonId) -> McsmResult<ProgressMap>;
    async fn env_image_platforms(&self, daemon: &DaemonId, body: &Value) -> McsmResult<Value>;

    async fn java_list(&self, daemon: &DaemonId) -> McsmResult<Vec<JavaRuntime>>;
    async fn java_add(&self, daemon: &DaemonId, body: &JavaAddBody) -> McsmResult<Value>;
    async fn java_download(&self, daemon: &DaemonId, body: &Value) -> McsmResult<Value>;
    async fn java_using(&self, daemon: &DaemonId, body: &Value) -> McsmResult<Value>;
    async fn java_delete(&self, daemon: &DaemonId, body: &Value) -> McsmResult<Value>;

    async fn mod_mc_versions(&self) -> McsmResult<Value>;
    async fn mod_list(&self, daemon: &DaemonId, uuid: &InstanceUuid) -> McsmResult<Value>;
    async fn mod_search(&self, q: &Value) -> McsmResult<Value>;
    async fn mod_download(
        &self,
        daemon: &DaemonId,
        uuid: &InstanceUuid,
        body: &ModDownloadBody,
    ) -> McsmResult<Value>;
    async fn mod_toggle(
        &self,
        daemon: &DaemonId,
        uuid: &InstanceUuid,
        body: &Value,
    ) -> McsmResult<Value>;
    async fn mod_delete(
        &self,
        daemon: &DaemonId,
        uuid: &InstanceUuid,
        body: &Value,
    ) -> McsmResult<Value>;

    async fn exchange(&self, body: &ExchangeRequest) -> McsmResult<Value>;
    async fn redeem(&self, body: &RedeemBody) -> McsmResult<Value>;

    /// Low-level escape hatch for endpoints not yet wrapped.
    async fn raw(
        &self,
        method: &str,
        path: &str,
        query: &[(String, String)],
        body: Option<&Value>,
    ) -> McsmResult<Value>;
}
