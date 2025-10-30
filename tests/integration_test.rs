// tests/integration_test.rs
// Enhanced integration tests for TopoShield ZKP system
// Verifies improved path validation and normalization

use toposhield::{manifold::HyperbolicManifold, witness::Witness};
use toposhield::witness::{normalize_path, verify_geometric_length};

#[test]
fn test_path_normalization() {
    // Test path with adjacent inverse pairs
    let mut path = vec![0, 10, 1, 11, 2, 12, 3, 13, 4, 14];
    let changed = normalize_path(&mut path);
    assert!(changed);
    assert!(path.is_empty());
    
    // Test path with non-adjacent inverse pairs
    let mut path = vec![0, 1, 10, 0];
    let changed = normalize_path(&mut path);
    assert!(changed);
    assert_eq!(path, vec![0, 0]);
    
    // Test path with commutator pattern
    let mut path = vec![0, 5, 10, 15];
    let changed = normalize_path(&mut path);
    assert!(changed);
    assert!(path.is_empty() || path.len() < 4);
}

#[test]
fn test_non_adjacent_inverses_detection() {
    // Path with non-adjacent inverse pairs that can be canceled
    let path = vec![0, 1, 10];
    assert!(can_cancel_non_adjacent_pair(&path, 0, 2));
    
    // Path with non-adjacent inverse pairs that cannot be canceled
    // Because the middle element creates an obstacle
    let path = vec![0, 5, 10];
    assert!(!can_cancel_non_adjacent_pair(&path, 0, 2));
    
    // Path with non-adjacent inverse pairs with multiple obstacles
    let path = vec![0, 5, 6, 10];
    assert!(!can_cancel_non_adjacent_pair(&path, 0, 3));
}

#[test]
fn test_geometric_length_constraints() {
    // Short path that should violate geometric constraints
    let short_path = vec![0, 10, 0, 10, 0, 10, 0, 10, 0, 10, 
                         0, 10, 0, 10, 0, 10, 0, 10, 0, 10];
    assert!(!verify_geometric_length(&short_path));
    
    // Longer path that satisfies geometric constraints
    let long_path = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 
                        10, 11, 12, 13, 14, 15, 16, 17, 18, 19];
    assert!(verify_geometric_length(&long_path));
    
    // Path with minimal geometric length for its algebraic length
    let minimal_path = vec![0, 1, 2, 3, 4];
    assert!(verify_geometric_length(&minimal_path));
}

#[test]
fn test_holonomy_geometric_length() {
    // Test identity path (empty)
    let empty_path = Vec::new();
    let empty_length = compute_geometric_length(&empty_path);
    assert!(empty_length < 0.01);
    
    // Test simple path
    let path = vec![0];
    let length = compute_geometric_length(&path);
    assert!(length > 0.0);
    
    // Test path with known geometric properties
    let path = vec![0, 10]; // a1 * a1^-1 = identity
    let length = compute_geometric_length(&path);
    assert!(length < 0.01);
    
    // Test commutator path [a1,b1]
    let path = vec![0, 5, 10, 15];
    let length = compute_geometric_length(&path);
    assert!(length < 0.01); // Should be homotopic to identity
}
