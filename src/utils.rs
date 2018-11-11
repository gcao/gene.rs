use uuid::Uuid;

pub fn new_uuidv4() -> String {
    return format!("{}", Uuid::new_v4());
}
