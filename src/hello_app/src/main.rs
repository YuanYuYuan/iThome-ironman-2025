use hello_derive::Hello;

// Define a trait the macro will implement
pub trait Hello {
    fn hello();
}

// Apply the custom derive
#[derive(Hello)]
struct Robot;

#[derive(Hello)]
struct Drone;

fn main() {
    Robot::hello(); // prints: Hello, I am Robot
    Drone::hello(); // prints: Hello, I am Drone
}
