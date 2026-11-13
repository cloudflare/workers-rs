use serde_json::{json, Value};
use worker::*;

const PROCESS_ENV: &str = r#"
const fs = require('node:fs');
const keys = ['BINDING_ALPHA', 'BINDING_EMPTY', '__proto__', 'BINDING_EXEC', 'PATH'];
const select = env => Object.fromEntries(keys.map(key => [key,
    Object.hasOwn(env, key) ? env[key] : null]));
const startup = Object.fromEntries(fs.readFileSync('/proc/1/environ', 'utf8')
    .split('\0').filter(Boolean).map(pair => {
        const index = pair.indexOf('=');
        return [pair.slice(0, index), pair.slice(index + 1)];
    }));
console.log(JSON.stringify({startup: select(startup), exec: select(process.env), cwd: process.cwd()}));
"#;

#[event(fetch)]
async fn fetch(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    let token = env.secret("REGRESSION_TOKEN")?.to_string();
    if req.method() != Method::Post
        || req.headers().get("Authorization")? != Some(format!("Bearer {token}"))
    {
        return Response::error("Unauthorized", 401);
    }
    let namespace = env.durable_object("CONTAINER_ENV")?;
    namespace
        .unique_id()?
        .get_stub()?
        .fetch_with_request(req)
        .await
}

#[durable_object]
pub struct ContainerEnvRegression {
    state: State,
}

impl DurableObject for ContainerEnvRegression {
    fn new(state: State, _env: Env) -> Self {
        Self { state }
    }

    async fn fetch(&self, _req: Request) -> Result<Response> {
        let container = self
            .state
            .container()
            .ok_or_else(|| Error::RustError("Container binding missing".into()))?;
        let mut startup = ContainerStartupOptions::new();
        startup.set_image("cloudflare/debian-trixie");
        startup.enable_internet(false);
        startup.set_entrypoint(&["/bin/sleep", "300"]);
        startup.add_env("BINDING_ALPHA", "startup spaces = quotes \" and λ\nnext");
        startup.add_env("BINDING_EMPTY", "");
        startup.add_env("__proto__", "startup ordinary key");
        startup.add_env(
            "PATH",
            "/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin:/binding-path-marker",
        );
        container.start(Some(startup))?;
        let result = probe(&container).await;
        container.destroy(None).await?;
        Response::from_json(&result?)
    }
}

async fn process_env(
    container: &Container,
    options: Option<ContainerExecOptions>,
) -> Result<Value> {
    let output = container
        .exec(&["node", "-e", PROCESS_ENV], options)
        .await?
        .output()
        .await?;
    if output.exit_code() != 0 || !output.stderr().is_empty() {
        return Err(Error::RustError(format!(
            "Environment reader failed: exit {}, stderr {}",
            output.exit_code(),
            String::from_utf8_lossy(&output.stderr())
        )));
    }
    Ok(serde_json::from_slice(&output.stdout())?)
}

async fn probe(container: &Container) -> Result<Value> {
    let baseline = process_env(container, None).await?;
    let mut options = ContainerExecOptions::new();
    options.add_env("BINDING_ALPHA", "exec override");
    options.add_env("BINDING_EXEC", "exec spaces = quotes \" and λ\nnext");
    options.add_env("BINDING_EMPTY", "");
    options.add_env("__proto__", "exec ordinary key");
    options.set_cwd("/tmp");
    options.set_user(0, 0);
    options.set_stdout("pipe");
    options.set_stderr("pipe");
    let overridden = process_env(container, Some(options)).await?;
    Ok(json!({"baseline": baseline, "overridden": overridden}))
}
