trait Animal {
    fn make_sound(&self);
}

struct Dog;

struct Cat;

impl Animal for Dog {
    fn make_sound(&self) {
        println!("Woof!");
    }
}

impl Animal for Cat {
    fn make_sound(&self) {
        println!("Meow!");
    }
}

fn main() {
    let dog = Dog;
    let cat = Cat;
    make_sound(&dog);
    make_sound(&cat);
}

fn make_sound<T: Animal>(animal: &T) {
    animal.make_sound();
}
