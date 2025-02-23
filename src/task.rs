pub trait Task {
    fn execute(&self);
}

pub struct PrintingTask;

impl Task for PrintingTask {
    fn execute(&self) {
        println!("Running printing Task... Goodbye")
    }
}
