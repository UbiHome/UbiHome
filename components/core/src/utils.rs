use convert_case::Case;
use convert_case::Casing;

/// Resolve the object id for a component.
///
/// A component must supply at least one of `id` / `name` (enforced during
/// validation). When an explicit `id` is given it always wins; otherwise the
/// `name` is normalized into snake_case. If neither is present (should not
/// happen for a validated component) an empty string is returned.
pub fn format_id(id: &Option<String>, name: &Option<String>) -> String {
    match id {
        Some(id) => id.clone(),
        None => name
            .as_ref()
            .map(|name| name.to_case(Case::Snake))
            .unwrap_or_default(),
    }
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
