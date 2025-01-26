trait Task {
    fn execute(&self);
}

trait A: Task {

    fn execute(&self) {
        self.activate();
        self.wait_duration();
        self.deactivate();
    }

    fn activate(&self);

    fn deactivate(&self);

    fn wait_duration(&self);
}
