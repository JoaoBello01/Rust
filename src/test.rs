#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    fn mock_database() -> UserDatabase {
        Arc::new(Mutex::new(HashMap::new()))
    }

    fn mock_user(cpf: &str) -> User {
        User {
            cpf: cpf.to_string(),
            full_name: "Test User".to_string(),
            email: "testuser@example.com".to_string(),
            birth: NaiveDate::from_ymd_opt(1995, 5, 15)
            .expect("Data inválida fornecida para 'birth'"),
            role: UserRole::User,
        }
    }

    #[tokio::test]
    async fn test_set_user_success() {
        let db = mock_database();
        let user = mock_user("12345678901");
        
        let result = set_user(&db, &user).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().cpf, "12345678901");
    }

    #[tokio::test]
    async fn test_set_user_duplicate_cpf() {
        let db = mock_database();
        let user = mock_user("12345678901");
        let _ = set_user(&db, &user).await; 
        
        let duplicate_result = set_user(&db, &user).await; 
        assert!(duplicate_result.is_err());
        assert_eq!(
            duplicate_result.unwrap_err().to_string(),
            "CPF já cadastrado!"
        );
    }

    #[tokio::test]
    async fn test_get_user_success() {
        let db = mock_database();
        let user = mock_user("12345678901");
        let _ = set_user(&db, &user).await;

        let fetched_user = get_user(&db, "12345678901").await;
        assert!(fetched_user.is_ok());
        assert_eq!(fetched_user.unwrap().cpf, "12345678901");
    }

    #[tokio::test]
    async fn test_get_user_not_found() {
        let db = mock_database();
        let fetched_user = get_user(&db, "99999999999").await;
        assert!(fetched_user.is_err());
    }

    #[tokio::test]
    async fn test_delete_user_success() {
        let db = mock_database();
        let user = mock_user("12345678901");
        let _ = set_user(&db, &user).await;

        let delete_result = delete_user(&db, "12345678901").await;
        assert!(delete_result.is_ok());

        let fetch_result = get_user(&db, "12345678901").await;
        assert!(fetch_result.is_err());
    }
}
