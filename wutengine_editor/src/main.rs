use wutengine::renderer::OpenGLRenderer;
use wutengine::runtime::RuntimeInitializer;

fn main() {
    let runtime = RuntimeInitializer::new();

    runtime.run::<OpenGLRenderer>();
}
