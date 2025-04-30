use convert_case::Case;
use convert_case::Casing;

pub fn format_id(id: &Option<String>, name: &String) -> String {
    id.clone().unwrap_or(name.clone().to_case(Case::Snake))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn use_id_if_specified() {
        let id = Some("id".to_string());
        let name = "name".to_string();

        let result = format_id(&id, &name);
        assert_eq!(result, "id");
    }

    #[test]
    fn generate_id_if_not_given() {
        let id = None;
        let name = "name".to_string();

        let result = format_id(&id, &name);
        assert_eq!(result, "name");
    }

    #[test]
    fn normalize_generated_id() {
        let id = None;
        let name = "Cool Sensor".to_string();

        let result = format_id(&id, &name);
        assert_eq!(result, "cool_sensor");
    }
}
