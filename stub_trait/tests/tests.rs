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
fn should_panic_if_stub_all_calls_of_is_called_twice() {
    let mut animal = StubAnimal::default();
    animal.stub_all_calls_of_name(|| "Ivana");
    animal.stub_all_calls_of_name(|| "Ivana");
}

#[test]
#[should_panic]
fn should_panic_if_stub_all_calls_of_is_called_after_register_stub_of() {
    let mut animal = StubAnimal::default();
    animal.register_stub_of_name(|| "Ivana");
    animal.stub_all_calls_of_name(|| "Ivana");
}

#[test]
#[should_panic]
fn should_panic_if_register_stub_of_is_called_after_stub_all_calls_of() {
    let mut animal = StubAnimal::default();
    animal.stub_all_calls_of_name(|| "Ivana");
    animal.register_stub_of_name(|| "Ivana");
}

#[test]
fn should_stub_all_calls() {
    let name = "Ivana";
    let mut animal = StubAnimal::default();
    animal.stub_all_calls_of_name(|| name);
    assert_eq!(animal.name(), name);
    assert_eq!(animal.name(), name);
    assert_eq!(animal.count_calls_of_name(), 2);
}

#[test]
fn should_stub_call_by_call() {
    let name1 = "Ivana";
    let name2 = "Truffle";
    let mut animal = StubAnimal::default();
    animal.register_stub_of_name(|| name1);
    animal.register_stub_of_name(|| name2);
    assert_eq!(animal.name(), name1);
    assert_eq!(animal.name(), name2);
    assert_eq!(animal.count_calls_of_name(), 2);
}
