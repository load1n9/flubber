use gdnative::api::MeshInstance;
use gdnative::export::hint::{EnumHint, IntHint, StringHint};
use gdnative::prelude::*;

use deno_core::error::AnyError;
use deno_core::include_js_files;
use deno_core::op;
use deno_core::url::Url;
use deno_core::Extension;
use deno_core::JsRuntime;
use deno_core::RuntimeOptions;
use deno_fetch::FetchPermissions;
use deno_web::BlobStore;
use deno_web::TimersPermission;

use std::path::Path;

struct AllowAllPermissions;

impl TimersPermission for AllowAllPermissions {
    fn allow_hrtime(&mut self) -> bool {
        true
    }

    fn check_unstable(&self, _state: &deno_core::OpState, _api_name: &'static str) {
        // allow unstable apis
    }
}

impl FetchPermissions for AllowAllPermissions {
    fn check_net_url(&mut self, _url: &Url, _api_name: &str) -> Result<(), AnyError> {
        Ok(())
    }

    fn check_read(&mut self, _path: &Path, _api_name: &str) -> Result<(), AnyError> {
        Ok(())
    }
}

// based on https://github.com/godot-rust/gdnative/tree/master/examples/spinning-cube
#[derive(gdnative::derive::NativeClass)]
#[inherit(MeshInstance)]
#[register_with(register_members)]
struct RustTest {
    start: Vector3,
    time: f32,
    #[property(path = "base/rotate_speed")]
    rotate_speed: f64,
}

fn register_members(builder: &ClassBuilder<RustTest>) {
    builder
        .property::<String>("test/test_enum")
        .with_hint(StringHint::Enum(EnumHint::new(vec![
            "Hello".into(),
            "World".into(),
            "Testing".into(),
        ])))
        .with_getter(|_: &RustTest, _| "Hello".to_string())
        .done();

    builder
        .property("test/test_flags")
        .with_hint(IntHint::Flags(EnumHint::new(vec![
            "A".into(),
            "B".into(),
            "C".into(),
            "D".into(),
        ])))
        .with_getter(|_: &RustTest, _| 0)
        .done();
}

#[methods]
impl RustTest {
    fn new(_owner: &MeshInstance) -> Self {
        RustTest {
            start: Vector3::new(0.0, 0.0, 0.0),
            time: 0.0,
            rotate_speed: 1.15,
        }
    }

    #[method]
    fn _ready(&mut self, #[base] owner: &MeshInstance) {
        owner.set_physics_process(true);
    }

    #[method]
    fn _physics_process(&mut self, #[base] owner: &MeshInstance, delta: f64) {
        use gdnative::api::SpatialMaterial;

        self.time += delta as f32;
        owner.rotate_y(self.rotate_speed * delta);

        let offset = Vector3::new(0.0, 1.0, 0.0) * self.time.cos() * 0.5;
        owner.set_translation(self.start + offset);

        if let Some(mat) = owner.get_surface_material(0) {
            let mat = unsafe { mat.assume_safe() };
            let mat = mat.cast::<SpatialMaterial>().expect("Incorrect material");
            mat.set_albedo(Color::from_rgba(self.time.cos().abs(), 0.0, 0.0, 1.0));
        }
    }
}
// based on https://github.com/lucacasonato/armadajs
fn init(handle: InitHandle) {
    handle.add_class::<RustTest>();

    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let path = "src/main.ts";
        let source_code = std::fs::read_to_string(&path).expect("entrypoint loaded");

        let flubber_ext = Extension::builder()
            .js(include_js_files!(
                prefix "flubber:",
                "core.js",
            ))
            .ops(vec![op_print::decl()])
            .state(|state| {
                state.put(AllowAllPermissions);
                Ok(())
            })
            .build();

        let mut runtime = JsRuntime::new(RuntimeOptions {
            extensions: vec![
                deno_console::init(),
                deno_webidl::init(),
                deno_url::init(),
                deno_web::init::<AllowAllPermissions>(BlobStore::default(), None),
                deno_fetch::init::<AllowAllPermissions>(deno_fetch::Options {
                    user_agent: format!("flubber/{}", env!("CARGO_PKG_VERSION")),
                    ..Default::default()
                }),
                flubber_ext,
            ],
            ..Default::default()
        });
        runtime
            .execute_script(&path, &source_code)
            .expect("execution succeeds");
        runtime
            .run_event_loop(false)
            .await
            .expect("execution succeeds");
    });
}

#[op]
fn op_print(value: String) {
    godot_print!("{}", value);
}

godot_init!(init);
