use blockchain_pow::application::Application;

fn main() {
    let mut app = Application::new(2);
    app.run();
}