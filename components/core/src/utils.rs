use convert_case::Case;
use convert_case::Casing;

/// Resolve the object id for a component: an explicit `id` wins, otherwise the
/// `name` is normalized into snake_case. At least one of them is guaranteed by
/// validation, so the absence of both is a bug rather than user error.
pub fn format_id(id: &Option<String>, name: &Option<String>) -> String {
    id.clone()
        .or_else(|| name.as_ref().map(|name| name.to_case(Case::Snake)))
        .expect("component must have a name or id")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn use_id_if_specified() {
        let id = Some("id".to_string());
        let name = Some("name".to_string());

        let result = format_id(&id, &name);
        assert_eq!(result, "id");
    }

    #[test]
    fn generate_id_if_not_given() {
        let id = None;
        let name = Some("name".to_string());

        let result = format_id(&id, &name);
        assert_eq!(result, "name");
    }

    #[test]
    fn normalize_generated_id() {
        let id = None;
        let name = Some("Cool Sensor".to_string());

        let result = format_id(&id, &name);
        assert_eq!(result, "cool_sensor");
    }

    #[test]
    fn use_id_when_name_is_absent() {
        let id = Some("internal_id".to_string());
        let name = None;

        let result = format_id(&id, &name);
        assert_eq!(result, "internal_id");
    }
}
