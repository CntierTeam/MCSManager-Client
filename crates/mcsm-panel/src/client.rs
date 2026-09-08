use crate::PanelClient;
use async_trait::async_trait;
use mcsm_protocol::auth::*;
use mcsm_protocol::environment::*;
use mcsm_protocol::error::{McsmError, McsmResult};
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
use mcsm_protocol::ApiEnvelope;
use reqwest::{Client, Method, RequestBuilder};
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::Value;
use std::time::Duration;

#[derive(Clone)]
pub struct HttpPanelClient {
    base: String,
    api_key: Option<String>,
    session_token: Option<String>,
    http: Client,
}

impl HttpPanelClient {
    pub fn new(panel_url: impl Into<String>) -> McsmResult<Self> {
        let base = panel_url.into().trim_end_matches('/').to_string();
        let http = Client::builder()
            .cookie_store(true)
            .timeout(Duration::from_secs(60))
            .user_agent(concat!("mcsm-cli/", env!("CARGO_PKG_VERSION")))
            .build()
            .map_err(|e| McsmError::Network(e.to_string()))?;
        Ok(Self {
            base,
            api_key: None,
            session_token: None,
            http,
        })
    }

    pub fn with_api_key(mut self, key: impl Into<String>) -> Self {
        let k = key.into();
        if !k.is_empty() {
            self.api_key = Some(k);
        }
        self
    }

    pub fn with_session_token(mut self, token: impl Into<String>) -> Self {
        let t = token.into();
        if !t.is_empty() {
            self.session_token = Some(t);
        }
        self
    }

    pub fn set_api_key_value(&mut self, key: Option<String>) {
        self.api_key = key.filter(|k| !k.is_empty());
    }

    pub fn set_session_token(&mut self, token: Option<String>) {
        self.session_token = token.filter(|t| !t.is_empty());
    }

    fn url(&self, path: &str) -> String {
        let path = if path.starts_with('/') {
            path.to_string()
        } else {
            format!("/{path}")
        };
        format!("{}{path}", self.base)
    }

    fn q(k: &str, v: impl Into<String>) -> (String, String) {
        (k.to_string(), v.into())
    }

    fn di(daemon: &DaemonId, uuid: &InstanceUuid) -> Vec<(String, String)> {
        vec![
            Self::q("daemonId", daemon.0.clone()),
            Self::q("uuid", uuid.0.clone()),
        ]
    }

    fn apply_auth(&self, mut rb: RequestBuilder) -> RequestBuilder {
        rb = rb.header("X-Requested-With", "XMLHttpRequest");
        rb = rb.header("Content-Type", "application/json");
        if let Some(key) = &self.api_key {
            rb = rb.header("X-Request-Api-Key", key);
            rb = rb.query(&[Self::q("apikey", key.clone())]);
        } else if let Some(token) = &self.session_token {
            rb = rb.query(&[Self::q("token", token.clone())]);
        }
        rb
    }

    fn parse_method(method: &str) -> McsmResult<Method> {
        match method.to_ascii_uppercase().as_str() {
            "GET" => Ok(Method::GET),
            "POST" => Ok(Method::POST),
            "PUT" => Ok(Method::PUT),
            "DELETE" => Ok(Method::DELETE),
            "PATCH" => Ok(Method::PATCH),
            "HEAD" => Ok(Method::HEAD),
            "OPTIONS" => Ok(Method::OPTIONS),
            other => Err(McsmError::Message(format!("unsupported HTTP method: {other}"))),
        }
    }

    async fn send_raw(
        &self,
        method: Method,
        path: &str,
        query: &[(String, String)],
        body: Option<&Value>,
    ) -> McsmResult<Value> {
        let mut rb = self.apply_auth(self.http.request(method, self.url(path)));
        if !query.is_empty() {
            rb = rb.query(query);
        }
        if let Some(b) = body {
            rb = rb.json(b);
        }
        let resp = rb
            .send()
            .await
            .map_err(|e| McsmError::Network(e.to_string()))?;
        let status_http = resp.status().as_u16() as i32;
        let text = resp
            .text()
            .await
            .map_err(|e| McsmError::Network(e.to_string()))?;
        if text.is_empty() {
            if (200..300).contains(&status_http) {
                return Ok(Value::Null);
            }
            return Err(McsmError::Api {
                status: status_http,
                message: "empty response".into(),
            });
        }
        let env: ApiEnvelope<Value> = serde_json::from_str(&text).map_err(|e| {
            McsmError::Serde(format!("envelope parse failed: {e}; body={text}"))
        })?;
        env.into_result_value()
    }

    async fn get_json<T: DeserializeOwned>(
        &self,
        path: &str,
        query: &[(String, String)],
    ) -> McsmResult<T> {
        let v = self.send_raw(Method::GET, path, query, None).await?;
        serde_json::from_value(v).map_err(|e| McsmError::Serde(e.to_string()))
    }

    async fn send_json<T: DeserializeOwned, B: Serialize>(
        &self,
        method: Method,
        path: &str,
        query: &[(String, String)],
        body: Option<&B>,
    ) -> McsmResult<T> {
        let body_v = match body {
            Some(b) => Some(serde_json::to_value(b).map_err(|e| McsmError::Serde(e.to_string()))?),
            None => None,
        };
        let v = self
            .send_raw(method, path, query, body_v.as_ref())
            .await?;
        serde_json::from_value(v).map_err(|e| McsmError::Serde(e.to_string()))
    }
}

#[async_trait]
impl PanelClient for HttpPanelClient {
    async fn login(&self, user: &str, pass: &str, code: Option<&str>) -> McsmResult<LoginOut> {
        let mut body = serde_json::json!({
            "username": user,
            "password": pass,
        });
        if let Some(c) = code {
            body["code"] = Value::String(c.to_string());
        }
        self.send_json(Method::POST, "/api/auth/login", &[], Some(&body))
            .await
    }

    async fn logout(&self) -> McsmResult<()> {
        let _: Value = self
            .send_json(Method::GET, "/api/auth/logout", &[], None::<&Value>)
            .await?;
        Ok(())
    }

    async fn status(&self) -> McsmResult<AuthStatus> {
        self.get_json("/api/auth/status", &[]).await
    }

    async fn login_info(&self) -> McsmResult<LoginPageInfo> {
        self.get_json("/api/auth/login_info", &[]).await
    }

    async fn install(&self, req: &InstallRequest) -> McsmResult<Value> {
        self.send_json(Method::POST, "/api/auth/install", &[], Some(req))
            .await
    }

    async fn me(&self) -> McsmResult<UserInfo> {
        self.get_json("/api/auth/", &[]).await
    }

    async fn update_profile(&self, req: &UpdatePasswordRequest) -> McsmResult<Value> {
        self.send_json(Method::PUT, "/api/auth/update", &[], Some(req))
            .await
    }

    async fn set_api_key(&self, enable: bool) -> McsmResult<Value> {
        let body = serde_json::json!({ "enable": enable });
        self.send_json(Method::PUT, "/api/auth/api", &[], Some(&body))
            .await
    }

    async fn bind2fa(&self) -> McsmResult<Value> {
        self.send_json(Method::POST, "/api/auth/bind2fa", &[], None::<&Value>)
            .await
    }

    async fn confirm2fa(&self, code: &str) -> McsmResult<Value> {
        let body = serde_json::json!({ "code": code });
        self.send_json(Method::POST, "/api/auth/confirm2fa", &[], Some(&body))
            .await
    }

    async fn create_user(&self, req: &CreateUserRequest) -> McsmResult<Value> {
        self.send_json(Method::POST, "/api/auth/", &[], Some(req))
            .await
    }

    async fn delete_user(&self, uuid: &str) -> McsmResult<Value> {
        let body = serde_json::json!([uuid]);
        self.send_json(Method::DELETE, "/api/auth/", &[], Some(&body))
            .await
    }

    async fn edit_user(&self, req: &EditUserRequest) -> McsmResult<Value> {
        self.send_json(Method::PUT, "/api/auth/", &[], Some(req))
            .await
    }

    async fn search_users(&self, q: &UserSearchQuery) -> McsmResult<UserPage> {
        let mut query = Vec::new();
        if let Some(n) = &q.userName {
            query.push(Self::q("userName", n.clone()));
        }
        if let Some(p) = q.page {
            query.push(Self::q("page", p.to_string()));
        }
        if let Some(ps) = q.page_size {
            query.push(Self::q("page_size", ps.to_string()));
        }
        self.get_json("/api/auth/search", &query).await
    }

    async fn user_overview(&self) -> McsmResult<Value> {
        self.get_json("/api/auth/overview", &[]).await
    }

    async fn query_username(&self, username: &str) -> McsmResult<Value> {
        let query = vec![Self::q("username", username)];
        self.get_json("/api/auth/query_username", &query).await
    }

    async fn sso_config(&self) -> McsmResult<Value> {
        self.get_json("/api/auth/sso/config", &[]).await
    }

    async fn overview(&self) -> McsmResult<Overview> {
        self.get_json("/api/overview", &[]).await
    }

    async fn operation_logs(&self) -> McsmResult<Value> {
        self.get_json("/api/overview/operation_logs", &[]).await
    }

    async fn search_operation_logs(&self, q: &AuditQuery) -> McsmResult<AuditPage> {
        let mut query = Vec::new();
        if let Some(v) = &q.type_filter {
            query.push(Self::q("type", v.clone()));
        }
        if let Some(v) = &q.level {
            query.push(Self::q("level", v.clone()));
        }
        if let Some(v) = &q.operator {
            query.push(Self::q("operator", v.clone()));
        }
        if let Some(v) = &q.keyword {
            query.push(Self::q("keyword", v.clone()));
        }
        if let Some(p) = q.page {
            query.push(Self::q("page", p.to_string()));
        }
        if let Some(ps) = q.page_size {
            query.push(Self::q("page_size", ps.to_string()));
        }
        self.get_json("/api/overview/operation_logs/search", &query)
            .await
    }

    async fn get_setting(&self) -> McsmResult<PanelSetting> {
        self.get_json("/api/overview/setting", &[]).await
    }

    async fn put_setting(&self, s: &PanelSetting) -> McsmResult<Value> {
        self.send_json(Method::PUT, "/api/overview/setting", &[], Some(s))
            .await
    }

    async fn get_layout(&self) -> McsmResult<LayoutData> {
        self.get_json("/api/overview/layout", &[]).await
    }

    async fn put_layout(&self, layout: &LayoutData) -> McsmResult<Value> {
        self.send_json(Method::POST, "/api/overview/layout", &[], Some(layout))
            .await
    }

    async fn delete_layout(&self) -> McsmResult<Value> {
        self.send_json(Method::DELETE, "/api/overview/layout", &[], None::<&Value>)
            .await
    }

    async fn install_language(&self, language: &str) -> McsmResult<Value> {
        let body = serde_json::json!({ "language": language });
        self.send_json(Method::PUT, "/api/overview/install", &[], Some(&body))
            .await
    }

    async fn list_nodes(&self) -> McsmResult<Vec<NodeSummary>> {
        self.get_json("/api/service/remote_services_list", &[])
            .await
    }

    async fn list_remote_services(&self) -> McsmResult<Value> {
        self.get_json("/api/service/remote_services", &[]).await
    }

    async fn remote_services_system(&self) -> McsmResult<Value> {
        self.get_json("/api/service/remote_services_system", &[])
            .await
    }

    async fn instances_on_node(&self, q: &InstanceListQuery) -> McsmResult<InstancePage> {
        let mut query = vec![
            Self::q("daemonId", q.daemonId.clone()),
            Self::q("page", q.page.to_string()),
            Self::q("page_size", q.page_size.to_string()),
        ];
        if let Some(n) = &q.instance_name {
            query.push(Self::q("instance_name", n.clone()));
        }
        if let Some(s) = &q.status {
            query.push(Self::q("status", s.clone()));
        }
        if let Some(t) = &q.tag {
            query.push(Self::q("tag", t.clone()));
        }
        self.get_json("/api/service/remote_service_instances", &query)
            .await
    }

    async fn instances_global(&self, q: &GlobalInstanceQuery) -> McsmResult<Value> {
        let mut query = vec![
            Self::q("page", q.page.to_string()),
            Self::q("page_size", q.page_size.to_string()),
        ];
        if let Some(n) = &q.instance_name {
            query.push(Self::q("instance_name", n.clone()));
        }
        if let Some(s) = &q.status {
            query.push(Self::q("status", s.clone()));
        }
        self.get_json("/api/service/remote_services_instances_global", &query)
            .await
    }

    async fn add_node(&self, req: &AddNode) -> McsmResult<Value> {
        self.send_json(Method::POST, "/api/service/remote_service", &[], Some(req))
            .await
    }

    async fn edit_node(&self, uuid: &DaemonId, req: &EditNode) -> McsmResult<Value> {
        let query = [Self::q("uuid", uuid.0.clone())];
        self.send_json(
            Method::PUT,
            "/api/service/remote_service",
            &query,
            Some(req),
        )
        .await
    }

    async fn delete_node(&self, uuid: &DaemonId) -> McsmResult<Value> {
        let query = [Self::q("uuid", uuid.0.clone())];
        self.send_json(
            Method::DELETE,
            "/api/service/remote_service",
            &query,
            None::<&Value>,
        )
        .await
    }

    async fn reconnect_node(&self, uuid: &DaemonId) -> McsmResult<Value> {
        let query = [Self::q("uuid", uuid.0.clone())];
        self.get_json("/api/service/link_remote_service", &query)
            .await
    }

    async fn instance_detail(
        &self,
        daemon: &DaemonId,
        uuid: &InstanceUuid,
    ) -> McsmResult<Instance> {
        let query = Self::di(daemon, uuid);
        self.get_json("/api/instance", &query).await
    }

    async fn create_instance(&self, daemon: &DaemonId, req: &CreateInstance) -> McsmResult<Value> {
        let query = [Self::q("daemonId", daemon.0.clone())];
        self.send_json(Method::POST, "/api/instance", &query, Some(req))
            .await
    }

    async fn update_instance(
        &self,
        daemon: &DaemonId,
        uuid: &InstanceUuid,
        req: &UpdateInstance,
    ) -> McsmResult<Value> {
        let query = Self::di(daemon, uuid);
        self.send_json(Method::PUT, "/api/instance", &query, Some(req))
            .await
    }

    async fn delete_instances(
        &self,
        daemon: &DaemonId,
        req: &DeleteInstances,
    ) -> McsmResult<Value> {
        let query = [Self::q("daemonId", daemon.0.clone())];
        self.send_json(Method::DELETE, "/api/instance", &query, Some(req))
            .await
    }

    async fn multi_open(&self, body: &Value) -> McsmResult<Value> {
        self.send_json(Method::POST, "/api/instance/multi_open", &[], Some(body))
            .await
    }

    async fn multi_stop(&self, body: &Value) -> McsmResult<Value> {
        self.send_json(Method::POST, "/api/instance/multi_stop", &[], Some(body))
            .await
    }

    async fn multi_kill(&self, body: &Value) -> McsmResult<Value> {
        self.send_json(Method::POST, "/api/instance/multi_kill", &[], Some(body))
            .await
    }

    async fn multi_restart(&self, body: &Value) -> McsmResult<Value> {
        self.send_json(Method::POST, "/api/instance/multi_restart", &[], Some(body))
            .await
    }

    async fn quick_install_list(&self) -> McsmResult<Value> {
        self.get_json("/api/instance/quick_install_list", &[]).await
    }

    async fn instance_upload_passport(
        &self,
        daemon: &DaemonId,
        upload_dir: &str,
    ) -> McsmResult<FilePassport> {
        let query = vec![
            Self::q("daemonId", daemon.0.clone()),
            Self::q("upload_dir", upload_dir),
        ];
        let v: Value = self
            .send_json(
                Method::POST,
                "/api/instance/upload",
                &query,
                None::<&Value>,
            )
            .await?;
        parse_file_passport(v)
    }

    async fn instance_open(&self, daemon: &DaemonId, uuid: &InstanceUuid) -> McsmResult<Value> {
        let query = Self::di(daemon, uuid);
        self.send_json(
            Method::GET,
            "/api/protected_instance/open",
            &query,
            None::<&Value>,
        )
        .await
    }

    async fn instance_stop(&self, daemon: &DaemonId, uuid: &InstanceUuid) -> McsmResult<Value> {
        let query = Self::di(daemon, uuid);
        self.send_json(
            Method::GET,
            "/api/protected_instance/stop",
            &query,
            None::<&Value>,
        )
        .await
    }

    async fn instance_restart(&self, daemon: &DaemonId, uuid: &InstanceUuid) -> McsmResult<Value> {
        let query = Self::di(daemon, uuid);
        self.send_json(
            Method::GET,
            "/api/protected_instance/restart",
            &query,
            None::<&Value>,
        )
        .await
    }

    async fn instance_kill(&self, daemon: &DaemonId, uuid: &InstanceUuid) -> McsmResult<Value> {
        let query = Self::di(daemon, uuid);
        self.send_json(
            Method::GET,
            "/api/protected_instance/kill",
            &query,
            None::<&Value>,
        )
        .await
    }

    async fn instance_command(
        &self,
        daemon: &DaemonId,
        uuid: &InstanceUuid,
        command: &str,
    ) -> McsmResult<Value> {
        let mut query = Self::di(daemon, uuid);
        query.push(Self::q("command", command));
        self.send_json(
            Method::GET,
            "/api/protected_instance/command",
            &query,
            None::<&Value>,
        )
        .await
    }

    async fn stream_channel(
        &self,
        daemon: &DaemonId,
        uuid: &InstanceUuid,
    ) -> McsmResult<StreamPassport> {
        let query = Self::di(daemon, uuid);
        let v: Value = self
            .send_json(
                Method::POST,
                "/api/protected_instance/stream_channel",
                &query,
                None::<&Value>,
            )
            .await?;
        serde_json::from_value(v).map_err(|e| McsmError::Serde(e.to_string()))
    }

    async fn outputlog(&self, daemon: &DaemonId, uuid: &InstanceUuid) -> McsmResult<String> {
        let query = Self::di(daemon, uuid);
        let v: Value = self
            .get_json("/api/protected_instance/outputlog", &query)
            .await?;
        match v {
            Value::String(s) => Ok(s),
            other => Ok(other.to_string()),
        }
    }

    async fn process_config_list(
        &self,
        daemon: &DaemonId,
        uuid: &InstanceUuid,
    ) -> McsmResult<Value> {
        let query = Self::di(daemon, uuid);
        let body = serde_json::json!({ "files": [] });
        self.send_json(
            Method::POST,
            "/api/protected_instance/process_config/list",
            &query,
            Some(&body),
        )
        .await
    }

    async fn process_config_get(
        &self,
        daemon: &DaemonId,
        uuid: &InstanceUuid,
        file_name: &str,
    ) -> McsmResult<Value> {
        let mut query = Self::di(daemon, uuid);
        query.push(Self::q("fileName", file_name));
        self.get_json("/api/protected_instance/process_config/file", &query)
            .await
    }

    async fn process_config_put(
        &self,
        daemon: &DaemonId,
        uuid: &InstanceUuid,
        file_name: &str,
        body: &Value,
    ) -> McsmResult<Value> {
        let mut query = Self::di(daemon, uuid);
        query.push(Self::q("fileName", file_name));
        self.send_json(
            Method::PUT,
            "/api/protected_instance/process_config/file",
            &query,
            Some(body),
        )
        .await
    }

    async fn instance_update_user(
        &self,
        daemon: &DaemonId,
        uuid: &InstanceUuid,
        body: &Value,
    ) -> McsmResult<Value> {
        let query = Self::di(daemon, uuid);
        self.send_json(
            Method::PUT,
            "/api/protected_instance/instance_update",
            &query,
            Some(body),
        )
        .await
    }

    async fn asynchronous(
        &self,
        daemon: &DaemonId,
        uuid: &InstanceUuid,
        task_name: &str,
        body: Option<&Value>,
    ) -> McsmResult<Value> {
        let mut query = Self::di(daemon, uuid);
        query.push(Self::q("task_name", task_name));
        self.send_json(
            Method::POST,
            "/api/protected_instance/asynchronous",
            &query,
            body,
        )
        .await
    }

    async fn query_asynchronous(
        &self,
        daemon: &DaemonId,
        uuid: &InstanceUuid,
        task_name: &str,
    ) -> McsmResult<Value> {
        let mut query = Self::di(daemon, uuid);
        query.push(Self::q("task_name", task_name));
        self.send_json(
            Method::GET,
            "/api/protected_instance/query_asynchronous",
            &query,
            None::<&Value>,
        )
        .await
    }

    async fn stop_asynchronous(
        &self,
        daemon: &DaemonId,
        uuid: &InstanceUuid,
        task_name: &str,
    ) -> McsmResult<Value> {
        let mut query = Self::di(daemon, uuid);
        query.push(Self::q("task_name", task_name));
        self.send_json(
            Method::GET,
            "/api/protected_instance/stop_asynchronous",
            &query,
            None::<&Value>,
        )
        .await
    }

    async fn install_instance(
        &self,
        daemon: &DaemonId,
        uuid: &InstanceUuid,
        body: &Value,
    ) -> McsmResult<Value> {
        let query = Self::di(daemon, uuid);
        self.send_json(
            Method::POST,
            "/api/protected_instance/install_instance",
            &query,
            Some(body),
        )
        .await
    }

    async fn file_status(&self, daemon: &DaemonId, uuid: &InstanceUuid) -> McsmResult<FileStatus> {
        let query = Self::di(daemon, uuid);
        self.get_json("/api/files/status", &query).await
    }

    async fn file_list(
        &self,
        daemon: &DaemonId,
        uuid: &InstanceUuid,
        q: &FileListQuery,
    ) -> McsmResult<FileList> {
        let mut query = Self::di(daemon, uuid);
        query.push(Self::q("target", q.target.clone()));
        query.push(Self::q("page", q.page.to_string()));
        query.push(Self::q("page_size", q.page_size.to_string()));
        if let Some(n) = &q.file_name {
            query.push(Self::q("file_name", n.clone()));
        }
        self.get_json("/api/files/list", &query).await
    }

    async fn file_edit(
        &self,
        daemon: &DaemonId,
        uuid: &InstanceUuid,
        body: &FileEditBody,
    ) -> McsmResult<Value> {
        let query = Self::di(daemon, uuid);
        self.send_json(Method::PUT, "/api/files", &query, Some(body))
            .await
    }

    async fn file_touch(
        &self,
        daemon: &DaemonId,
        uuid: &InstanceUuid,
        target: &str,
    ) -> McsmResult<Value> {
        let body = serde_json::json!({ "target": target });
        let query = Self::di(daemon, uuid);
        self.send_json(Method::POST, "/api/files/touch", &query, Some(&body))
            .await
    }

    async fn file_mkdir(
        &self,
        daemon: &DaemonId,
        uuid: &InstanceUuid,
        target: &str,
    ) -> McsmResult<Value> {
        let body = serde_json::json!({ "target": target });
        let query = Self::di(daemon, uuid);
        self.send_json(Method::POST, "/api/files/mkdir", &query, Some(&body))
            .await
    }

    async fn file_copy(
        &self,
        daemon: &DaemonId,
        uuid: &InstanceUuid,
        body: &Value,
    ) -> McsmResult<Value> {
        let query = Self::di(daemon, uuid);
        self.send_json(Method::POST, "/api/files/copy", &query, Some(body))
            .await
    }

    async fn file_move(
        &self,
        daemon: &DaemonId,
        uuid: &InstanceUuid,
        body: &Value,
    ) -> McsmResult<Value> {
        let query = Self::di(daemon, uuid);
        self.send_json(Method::PUT, "/api/files/move", &query, Some(body))
            .await
    }

    async fn file_delete(
        &self,
        daemon: &DaemonId,
        uuid: &InstanceUuid,
        body: &Value,
    ) -> McsmResult<Value> {
        let query = Self::di(daemon, uuid);
        self.send_json(Method::DELETE, "/api/files", &query, Some(body))
            .await
    }

    async fn file_chmod(
        &self,
        daemon: &DaemonId,
        uuid: &InstanceUuid,
        body: &FileChmodBody,
    ) -> McsmResult<Value> {
        let query = Self::di(daemon, uuid);
        self.send_json(Method::PUT, "/api/files/chmod", &query, Some(body))
            .await
    }

    async fn file_compress(
        &self,
        daemon: &DaemonId,
        uuid: &InstanceUuid,
        body: &Value,
    ) -> McsmResult<Value> {
        let query = Self::di(daemon, uuid);
        self.send_json(Method::POST, "/api/files/compress", &query, Some(body))
            .await
    }

    async fn file_download_from_url(
        &self,
        daemon: &DaemonId,
        uuid: &InstanceUuid,
        body: &DownloadFromUrlBody,
    ) -> McsmResult<Value> {
        let query = Self::di(daemon, uuid);
        self.send_json(
            Method::POST,
            "/api/files/download_from_url",
            &query,
            Some(body),
        )
        .await
    }

    async fn file_download_passport(
        &self,
        daemon: &DaemonId,
        uuid: &InstanceUuid,
        file_name: &str,
    ) -> McsmResult<FilePassport> {
        let mut query = Self::di(daemon, uuid);
        query.push(Self::q("file_name", file_name));
        let v: Value = self
            .send_json(Method::POST, "/api/files/download", &query, None::<&Value>)
            .await?;
        parse_file_passport(v)
    }

    async fn file_upload_passport(
        &self,
        daemon: &DaemonId,
        uuid: &InstanceUuid,
        upload_dir: &str,
    ) -> McsmResult<FilePassport> {
        let mut query = Self::di(daemon, uuid);
        query.push(Self::q("upload_dir", upload_dir));
        let v: Value = self
            .send_json(Method::POST, "/api/files/upload", &query, None::<&Value>)
            .await?;
        parse_file_passport(v)
    }

    async fn schedule_list(
        &self,
        daemon: &DaemonId,
        uuid: &InstanceUuid,
    ) -> McsmResult<Vec<ScheduleTask>> {
        let query = Self::di(daemon, uuid);
        self.get_json("/api/protected_schedule", &query).await
    }

    async fn schedule_create(
        &self,
        daemon: &DaemonId,
        uuid: &InstanceUuid,
        body: &CreateSchedule,
    ) -> McsmResult<Value> {
        let query = Self::di(daemon, uuid);
        self.send_json(Method::POST, "/api/protected_schedule", &query, Some(body))
            .await
    }

    async fn schedule_delete(
        &self,
        daemon: &DaemonId,
        uuid: &InstanceUuid,
        name: &str,
    ) -> McsmResult<Value> {
        let query = Self::di(daemon, uuid);
        self.send_json(
            Method::DELETE,
            "/api/protected_schedule",
            &query,
            Some(&serde_json::json!({ "name": name })),
        )
        .await
    }

    async fn env_images(&self, daemon: &DaemonId) -> McsmResult<Vec<DockerImage>> {
        let query = [Self::q("daemonId", daemon.0.clone())];
        self.get_json("/api/environment/image", &query).await
    }

    async fn env_new_image(&self, daemon: &DaemonId, body: &NewImageBody) -> McsmResult<Value> {
        let query = [Self::q("daemonId", daemon.0.clone())];
        self.send_json(Method::POST, "/api/environment/image", &query, Some(body))
            .await
    }

    async fn env_del_image(&self, daemon: &DaemonId, image_id: &str) -> McsmResult<Value> {
        let query = vec![
            Self::q("daemonId", daemon.0.clone()),
            Self::q("imageId", image_id),
        ];
        self.send_json(
            Method::DELETE,
            "/api/environment/image",
            &query,
            None::<&Value>,
        )
        .await
    }

    async fn env_containers(&self, daemon: &DaemonId) -> McsmResult<Value> {
        let query = [Self::q("daemonId", daemon.0.clone())];
        self.get_json("/api/environment/containers", &query).await
    }

    async fn env_network_modes(&self, daemon: &DaemonId) -> McsmResult<Value> {
        let query = [Self::q("daemonId", daemon.0.clone())];
        self.get_json("/api/environment/networkModes", &query)
            .await
    }

    async fn env_progress(&self, daemon: &DaemonId) -> McsmResult<ProgressMap> {
        let query = [Self::q("daemonId", daemon.0.clone())];
        self.get_json("/api/environment/progress", &query).await
    }

    async fn env_image_platforms(&self, daemon: &DaemonId, body: &Value) -> McsmResult<Value> {
        let query = [Self::q("daemonId", daemon.0.clone())];
        self.send_json(
            Method::POST,
            "/api/environment/image_platforms",
            &query,
            Some(body),
        )
        .await
    }

    async fn java_list(&self, daemon: &DaemonId) -> McsmResult<Vec<JavaRuntime>> {
        let query = [Self::q("daemonId", daemon.0.clone())];
        self.get_json("/api/java_manager/list", &query).await
    }

    async fn java_add(&self, daemon: &DaemonId, body: &JavaAddBody) -> McsmResult<Value> {
        let query = [Self::q("daemonId", daemon.0.clone())];
        self.send_json(Method::POST, "/api/java_manager/add", &query, Some(body))
            .await
    }

    async fn java_download(&self, daemon: &DaemonId, body: &Value) -> McsmResult<Value> {
        let query = [Self::q("daemonId", daemon.0.clone())];
        self.send_json(
            Method::POST,
            "/api/java_manager/download",
            &query,
            Some(body),
        )
        .await
    }

    async fn java_using(&self, daemon: &DaemonId, body: &Value) -> McsmResult<Value> {
        let query = [Self::q("daemonId", daemon.0.clone())];
        self.send_json(Method::POST, "/api/java_manager/using", &query, Some(body))
            .await
    }

    async fn java_delete(&self, daemon: &DaemonId, body: &Value) -> McsmResult<Value> {
        let query = [Self::q("daemonId", daemon.0.clone())];
        self.send_json(
            Method::DELETE,
            "/api/java_manager/delete",
            &query,
            Some(body),
        )
        .await
    }

    async fn mod_mc_versions(&self) -> McsmResult<Value> {
        self.get_json("/api/mod/mc_versions", &[]).await
    }

    async fn mod_list(&self, daemon: &DaemonId, uuid: &InstanceUuid) -> McsmResult<Value> {
        let query = Self::di(daemon, uuid);
        self.get_json("/api/mod/list", &query).await
    }

    async fn mod_search(&self, q: &Value) -> McsmResult<Value> {
        let query: Vec<(String, String)> = q
            .as_object()
            .map(|m| {
                m.iter()
                    .map(|(k, v)| {
                        Self::q(
                            k,
                            match v {
                                Value::String(s) => s.clone(),
                                other => other.to_string(),
                            },
                        )
                    })
                    .collect()
            })
            .unwrap_or_default();
        self.get_json("/api/mod/search", &query).await
    }

    async fn mod_download(
        &self,
        daemon: &DaemonId,
        uuid: &InstanceUuid,
        body: &ModDownloadBody,
    ) -> McsmResult<Value> {
        let query = Self::di(daemon, uuid);
        self.send_json(Method::POST, "/api/mod/download", &query, Some(body))
            .await
    }

    async fn mod_toggle(
        &self,
        daemon: &DaemonId,
        uuid: &InstanceUuid,
        body: &Value,
    ) -> McsmResult<Value> {
        let query = Self::di(daemon, uuid);
        self.send_json(Method::POST, "/api/mod/toggle", &query, Some(body))
            .await
    }

    async fn mod_delete(
        &self,
        daemon: &DaemonId,
        uuid: &InstanceUuid,
        body: &Value,
    ) -> McsmResult<Value> {
        let query = Self::di(daemon, uuid);
        self.send_json(Method::POST, "/api/mod/delete", &query, Some(body))
            .await
    }

    async fn exchange(&self, body: &ExchangeRequest) -> McsmResult<Value> {
        self.send_json(Method::POST, "/api/exchange/", &[], Some(body))
            .await
    }

    async fn redeem(&self, body: &RedeemBody) -> McsmResult<Value> {
        self.send_json(
            Method::POST,
            "/api/exchange/request_buy_instance",
            &[],
            Some(body),
        )
        .await
    }

    async fn raw(
        &self,
        method: &str,
        path: &str,
        query: &[(String, String)],
        body: Option<&Value>,
    ) -> McsmResult<Value> {
        let method = Self::parse_method(method)?;
        self.send_raw(method, path, query, body).await
    }
}

fn parse_file_passport(v: Value) -> McsmResult<FilePassport> {
    match v {
        Value::String(url) => Ok(FilePassport {
            url,
            ..Default::default()
        }),
        Value::Object(map) => {
            let password = map
                .get("password")
                .and_then(|x| x.as_str())
                .map(|s| s.to_string());
            let addr = map
                .get("addr")
                .and_then(|x| x.as_str())
                .map(|s| s.to_string());
            let prefix = map
                .get("prefix")
                .and_then(|x| x.as_str())
                .map(|s| s.to_string());
            let url = map
                .get("url")
                .and_then(|x| x.as_str())
                .unwrap_or("")
                .to_string();
            Ok(FilePassport {
                url,
                password,
                addr,
                prefix,
            })
        }
        other => Err(McsmError::Serde(format!(
            "unexpected file passport: {other}"
        ))),
    }
}
