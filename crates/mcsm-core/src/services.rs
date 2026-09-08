use crate::AppContext;
use mcsm_daemon::{DefaultDaemonClient, DaemonClient, StreamSession};
use mcsm_protocol::auth::*;
use mcsm_protocol::error::McsmResult;
use mcsm_protocol::files::FileListQuery;
use mcsm_protocol::ids::{DaemonId, InstanceUuid};
use mcsm_protocol::overview::*;
use mcsm_protocol::schedule::CreateSchedule;
use mcsm_protocol::service::*;
use mcsm_protocol::stream::StreamPassport;
use serde_json::Value;
use std::path::Path;

pub struct AuthService<'a>(pub &'a AppContext);
pub struct OverviewService<'a>(pub &'a AppContext);
pub struct NodeService<'a>(pub &'a AppContext);
pub struct InstanceService<'a>(pub &'a AppContext);
pub struct TerminalService<'a>(pub &'a AppContext);
pub struct FileService<'a>(pub &'a AppContext);
pub struct ScheduleService<'a>(pub &'a AppContext);
pub struct UserService<'a>(pub &'a AppContext);
pub struct SettingsService<'a>(pub &'a AppContext);
pub struct AuditService<'a>(pub &'a AppContext);
pub struct MarketService<'a>(pub &'a AppContext);
pub struct JavaService<'a>(pub &'a AppContext);
pub struct ModService<'a>(pub &'a AppContext);
pub struct EnvService<'a>(pub &'a AppContext);
pub struct ExchangeService<'a>(pub &'a AppContext);

impl AuthService<'_> {
    pub async fn status(&self) -> McsmResult<AuthStatus> {
        self.0.panel.status().await
    }
    pub async fn login(&self, user: &str, pass: &str, code: Option<&str>) -> McsmResult<LoginOut> {
        self.0.panel.login(user, pass, code).await
    }
    pub async fn me(&self) -> McsmResult<UserInfo> {
        self.0.panel.me().await
    }
    pub async fn set_api_key(&self, enable: bool) -> McsmResult<Value> {
        self.0.panel.set_api_key(enable).await
    }
    pub async fn bind2fa(&self) -> McsmResult<Value> {
        self.0.panel.bind2fa().await
    }
    pub async fn confirm2fa(&self, code: &str) -> McsmResult<Value> {
        self.0.panel.confirm2fa(code).await
    }
}

impl OverviewService<'_> {
    pub async fn get(&self) -> McsmResult<Overview> {
        self.0.panel.overview().await
    }
}

impl NodeService<'_> {
    pub async fn list(&self) -> McsmResult<Vec<NodeSummary>> {
        self.0.panel.list_nodes().await
    }
    pub async fn add(&self, req: &AddNode) -> McsmResult<Value> {
        self.0.panel.add_node(req).await
    }
    pub async fn edit(&self, uuid: &DaemonId, req: &EditNode) -> McsmResult<Value> {
        self.0.panel.edit_node(uuid, req).await
    }
    pub async fn delete(&self, uuid: &DaemonId) -> McsmResult<Value> {
        self.0.panel.delete_node(uuid).await
    }
    pub async fn reconnect(&self, uuid: &DaemonId) -> McsmResult<Value> {
        self.0.panel.reconnect_node(uuid).await
    }
    pub async fn system(&self) -> McsmResult<Value> {
        self.0.panel.remote_services_system().await
    }
}

impl InstanceService<'_> {
    pub async fn list_on_node(&self, q: &InstanceListQuery) -> McsmResult<InstancePage> {
        self.0.panel.instances_on_node(q).await
    }
    pub async fn list_global(&self, q: &GlobalInstanceQuery) -> McsmResult<Value> {
        self.0.panel.instances_global(q).await
    }
    pub async fn get(&self, daemon: &DaemonId, uuid: &InstanceUuid) -> McsmResult<mcsm_protocol::instance::Instance> {
        self.0.panel.instance_detail(daemon, uuid).await
    }
    pub async fn open(&self, daemon: &DaemonId, uuid: &InstanceUuid) -> McsmResult<Value> {
        self.0.panel.instance_open(daemon, uuid).await
    }
    pub async fn stop(&self, daemon: &DaemonId, uuid: &InstanceUuid) -> McsmResult<Value> {
        self.0.panel.instance_stop(daemon, uuid).await
    }
    pub async fn restart(&self, daemon: &DaemonId, uuid: &InstanceUuid) -> McsmResult<Value> {
        self.0.panel.instance_restart(daemon, uuid).await
    }
    pub async fn kill(&self, daemon: &DaemonId, uuid: &InstanceUuid) -> McsmResult<Value> {
        self.0.panel.instance_kill(daemon, uuid).await
    }
    pub async fn command(
        &self,
        daemon: &DaemonId,
        uuid: &InstanceUuid,
        cmd: &str,
    ) -> McsmResult<Value> {
        self.0.panel.instance_command(daemon, uuid, cmd).await
    }
    pub async fn create(
        &self,
        daemon: &DaemonId,
        req: &mcsm_protocol::instance::CreateInstance,
    ) -> McsmResult<Value> {
        self.0.panel.create_instance(daemon, req).await
    }
    pub async fn update(
        &self,
        daemon: &DaemonId,
        uuid: &InstanceUuid,
        req: &mcsm_protocol::instance::UpdateInstance,
    ) -> McsmResult<Value> {
        self.0.panel.update_instance(daemon, uuid, req).await
    }
    pub async fn delete(
        &self,
        daemon: &DaemonId,
        req: &mcsm_protocol::instance::DeleteInstances,
    ) -> McsmResult<Value> {
        self.0.panel.delete_instances(daemon, req).await
    }
    pub async fn multi_open(&self, body: &Value) -> McsmResult<Value> {
        self.0.panel.multi_open(body).await
    }
    pub async fn multi_stop(&self, body: &Value) -> McsmResult<Value> {
        self.0.panel.multi_stop(body).await
    }
    pub async fn multi_kill(&self, body: &Value) -> McsmResult<Value> {
        self.0.panel.multi_kill(body).await
    }
    pub async fn multi_restart(&self, body: &Value) -> McsmResult<Value> {
        self.0.panel.multi_restart(body).await
    }
}

impl TerminalService<'_> {
    pub async fn stream_channel(
        &self,
        daemon: &DaemonId,
        uuid: &InstanceUuid,
    ) -> McsmResult<StreamPassport> {
        self.0.panel.stream_channel(daemon, uuid).await
    }

    pub async fn attach(
        &self,
        daemon: &DaemonId,
        uuid: &InstanceUuid,
    ) -> McsmResult<StreamSession> {
        let passport = self.stream_channel(daemon, uuid).await?;
        DefaultDaemonClient::connect_stream(passport).await
    }

    pub async fn outputlog(&self, daemon: &DaemonId, uuid: &InstanceUuid) -> McsmResult<String> {
        self.0.panel.outputlog(daemon, uuid).await
    }
}

impl FileService<'_> {
    pub async fn list(
        &self,
        daemon: &DaemonId,
        uuid: &InstanceUuid,
        q: &FileListQuery,
    ) -> McsmResult<mcsm_protocol::files::FileList> {
        self.0.panel.file_list(daemon, uuid, q).await
    }
    pub async fn mkdir(
        &self,
        daemon: &DaemonId,
        uuid: &InstanceUuid,
        target: &str,
    ) -> McsmResult<Value> {
        self.0.panel.file_mkdir(daemon, uuid, target).await
    }
    pub async fn touch(
        &self,
        daemon: &DaemonId,
        uuid: &InstanceUuid,
        target: &str,
    ) -> McsmResult<Value> {
        self.0.panel.file_touch(daemon, uuid, target).await
    }
    pub async fn delete(&self, daemon: &DaemonId, uuid: &InstanceUuid, body: &Value) -> McsmResult<Value> {
        self.0.panel.file_delete(daemon, uuid, body).await
    }
    pub async fn edit(
        &self,
        daemon: &DaemonId,
        uuid: &InstanceUuid,
        body: &mcsm_protocol::files::FileEditBody,
    ) -> McsmResult<Value> {
        self.0.panel.file_edit(daemon, uuid, body).await
    }
    pub async fn upload(
        &self,
        daemon: &DaemonId,
        uuid: &InstanceUuid,
        upload_dir: &str,
        local: &Path,
    ) -> McsmResult<()> {
        let passport = self
            .0
            .panel
            .file_upload_passport(daemon, uuid, upload_dir)
            .await?;
        let url = passport.upload_url()?;
        DefaultDaemonClient::upload(&url, local).await
    }
    pub async fn download(
        &self,
        daemon: &DaemonId,
        uuid: &InstanceUuid,
        remote_name: &str,
        dest: &Path,
    ) -> McsmResult<()> {
        let passport = self
            .0
            .panel
            .file_download_passport(daemon, uuid, remote_name)
            .await?;
        let url = passport.download_url(remote_name)?;
        DefaultDaemonClient::download(&url, dest).await
    }
}

impl ScheduleService<'_> {
    pub async fn list(
        &self,
        daemon: &DaemonId,
        uuid: &InstanceUuid,
    ) -> McsmResult<Vec<mcsm_protocol::schedule::ScheduleTask>> {
        self.0.panel.schedule_list(daemon, uuid).await
    }
    pub async fn create(
        &self,
        daemon: &DaemonId,
        uuid: &InstanceUuid,
        body: &CreateSchedule,
    ) -> McsmResult<Value> {
        self.0.panel.schedule_create(daemon, uuid, body).await
    }
    pub async fn delete(
        &self,
        daemon: &DaemonId,
        uuid: &InstanceUuid,
        name: &str,
    ) -> McsmResult<Value> {
        self.0.panel.schedule_delete(daemon, uuid, name).await
    }
}

impl UserService<'_> {
    pub async fn search(&self, q: &UserSearchQuery) -> McsmResult<UserPage> {
        self.0.panel.search_users(q).await
    }
    pub async fn create(&self, req: &CreateUserRequest) -> McsmResult<Value> {
        self.0.panel.create_user(req).await
    }
    pub async fn edit(&self, req: &EditUserRequest) -> McsmResult<Value> {
        self.0.panel.edit_user(req).await
    }
    pub async fn delete(&self, uuid: &str) -> McsmResult<Value> {
        self.0.panel.delete_user(uuid).await
    }
}

impl SettingsService<'_> {
    pub async fn get(&self) -> McsmResult<PanelSetting> {
        self.0.panel.get_setting().await
    }
    pub async fn put(&self, s: &PanelSetting) -> McsmResult<Value> {
        self.0.panel.put_setting(s).await
    }
}

impl AuditService<'_> {
    pub async fn search(&self, q: &AuditQuery) -> McsmResult<AuditPage> {
        self.0.panel.search_operation_logs(q).await
    }
    pub async fn recent(&self) -> McsmResult<Value> {
        self.0.panel.operation_logs().await
    }
}

impl MarketService<'_> {
    pub async fn quick_install_list(&self) -> McsmResult<Value> {
        self.0.panel.quick_install_list().await
    }
    pub async fn install(
        &self,
        daemon: &DaemonId,
        uuid: &InstanceUuid,
        body: &Value,
    ) -> McsmResult<Value> {
        self.0.panel.install_instance(daemon, uuid, body).await
    }
}

impl JavaService<'_> {
    pub async fn list(&self, daemon: &DaemonId) -> McsmResult<Vec<mcsm_protocol::java::JavaRuntime>> {
        self.0.panel.java_list(daemon).await
    }
}

impl ModService<'_> {
    pub async fn list(&self, daemon: &DaemonId, uuid: &InstanceUuid) -> McsmResult<Value> {
        self.0.panel.mod_list(daemon, uuid).await
    }
    pub async fn search(&self, q: &Value) -> McsmResult<Value> {
        self.0.panel.mod_search(q).await
    }
}

impl EnvService<'_> {
    pub async fn images(
        &self,
        daemon: &DaemonId,
    ) -> McsmResult<Vec<mcsm_protocol::environment::DockerImage>> {
        self.0.panel.env_images(daemon).await
    }
    pub async fn containers(&self, daemon: &DaemonId) -> McsmResult<Value> {
        self.0.panel.env_containers(daemon).await
    }
}

impl ExchangeService<'_> {
    pub async fn exchange(
        &self,
        body: &mcsm_protocol::exchange::ExchangeRequest,
    ) -> McsmResult<Value> {
        self.0.panel.exchange(body).await
    }
    pub async fn redeem(&self, body: &mcsm_protocol::exchange::RedeemBody) -> McsmResult<Value> {
        self.0.panel.redeem(body).await
    }
}
