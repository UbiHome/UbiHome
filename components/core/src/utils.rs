use convert_case::Case;
use convert_case::Casing;

pub fn format_id(base_name: &String, id: &Option<String>, name: &String) -> String {
    id.clone().unwrap_or(format!(
        "{}_{}",
        base_name,
        name.clone().to_case(Case::Snake)
    ))
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn use_id_if_specified() {
        let base_name = "base".to_string();
        let id = Some("id".to_string());
        let name = "name".to_string();

        let result = format_id(&base_name, &id, &name);
        assert_eq!(result, "id");
    }

    #[test]
    fn generate_id_if_not_given() {
        let base_name = "base".to_string();
        let id = None;
        let name = "name".to_string();

        let result = format_id(&base_name, &id, &name);
        assert_eq!(result, "base_name");
    }

    #[test]
    fn normalize_generated_id() {
        let base_name = "base".to_string();
        let id = None;
        let name = "Cool Sensor".to_string();

        let result = format_id(&base_name, &id, &name);
        assert_eq!(result, "base_cool_sensor");
    }
}