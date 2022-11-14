use stub_trait::stub;

#[stub]
trait Animal {
    fn name(&self) -> &str;
}

#[stub]
trait List<T> {
    fn add(&mut self, item: T);
}

#[stub]
trait Person<'a> {
    fn name(&self) -> &'a str;
}

#[test]
#[should_panic]
fn panic_if_unexpected_invocation() {
    let animal = StubAnimal::new();
    animal.name();
}
