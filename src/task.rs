pub trait Task: Send {
    fn execute(&self);
}

pub struct PrintingTask(String);

impl PrintingTask {
    pub fn new(text: String) -> Self {
        Self(text)
    }
}

impl Task for PrintingTask {
    fn execute(&self) {
        println!("{}", self.0);
    }
}
