//! Lumina Standard Library (LSL) — Virtual Schema Registry
//!
//! Provides pre-defined entity schemas for common datacenter, network,
//! and infrastructure components. These are composed (not inherited)
//! into user-defined entities via `import LSL::namespace::Entity`.

use lumina_lexer::token::Span;
use lumina_parser::ast::{EntityDecl, Field, FieldMetadata, LuminaType, StoredField};
use rustc_hash::FxHashMap;

/// The LSL registry: a virtual filesystem of pre-defined entity schemas.
pub struct LslRegistry {
    schemas: FxHashMap<String, EntityDecl>,
}

impl LslRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            schemas: FxHashMap::default(),
        };
        registry.register_datacenter();
        registry.register_network();

        registry.register_power();
        registry.register_docker();
        registry
    }

    /// Look up a schema by its full LSL path, e.g. "LSL::datacenter::Server"
    pub fn resolve(&self, path: &str) -> Option<&EntityDecl> {
        self.schemas.get(path)
    }

    /// Return all registered schema names
    pub fn list_schemas(&self) -> Vec<&str> {
        let mut names: Vec<&str> = self.schemas.keys().map(|s| s.as_str()).collect();
        names.sort();
        names
    }

    /// Get the field definitions for a given LSL schema
    pub fn get_fields(&self, path: &str) -> Option<&Vec<Field>> {
        self.schemas.get(path).map(|e| &e.fields)
    }

    // ── Datacenter schemas ─────────────────────────────────

    fn register_datacenter(&mut self) {
        let span = Span {
            start: 0,
            end: 0,
            line: 0,
            col: 0,
        };
        let meta = FieldMetadata {
            doc: None,
            range: None,
            affects: vec![],
        };

        // LSL::datacenter::Server
        self.schemas.insert(
            "LSL::datacenter::Server".into(),
            EntityDecl {
                name: "Server".into(),
                fields: vec![
                    Field::Stored(StoredField {
                        name: "hostname".into(),
                        ty: LuminaType::Text,
                        metadata: meta.clone(),
                        span,
                    }),
                    Field::Stored(StoredField {
                        name: "mgmt_ip".into(),
                        ty: LuminaType::Text,
                        metadata: meta.clone(),
                        span,
                    }),
                    Field::Stored(StoredField {
                        name: "cpu_percent".into(),
                        ty: LuminaType::Number,
                        metadata: FieldMetadata {
                            doc: Some("CPU utilization 0-100".into()),
                            range: Some((0.0, 100.0)),
                            affects: vec![],
                        },
                        span,
                    }),
                    Field::Stored(StoredField {
                        name: "memory_percent".into(),
                        ty: LuminaType::Number,
                        metadata: FieldMetadata {
                            doc: Some("Memory utilization 0-100".into()),
                            range: Some((0.0, 100.0)),
                            affects: vec![],
                        },
                        span,
                    }),
                    Field::Stored(StoredField {
                        name: "disk_percent".into(),
                        ty: LuminaType::Number,
                        metadata: meta.clone(),
                        span,
                    }),
                    Field::Stored(StoredField {
                        name: "inlet_temp_c".into(),
                        ty: LuminaType::Number,
                        metadata: FieldMetadata {
                            doc: Some("Inlet temperature in Celsius".into()),
                            range: Some((0.0, 85.0)),
                            affects: vec![],
                        },
                        span,
                    }),
                    Field::Stored(StoredField {
                        name: "power_watts".into(),
                        ty: LuminaType::Number,
                        metadata: meta.clone(),
                        span,
                    }),
                    Field::Stored(StoredField {
                        name: "status".into(),
                        ty: LuminaType::Text,
                        metadata: meta.clone(),
                        span,
                    }),
                    Field::Stored(StoredField {
                        name: "model".into(),
                        ty: LuminaType::Text,
                        metadata: meta.clone(),
                        span,
                    }),
                    Field::Stored(StoredField {
                        name: "serial_number".into(),
                        ty: LuminaType::Text,
                        metadata: meta.clone(),
                        span,
                    }),
                ],
                span,
            },
        );

        // LSL::datacenter::Rack
        self.schemas.insert(
            "LSL::datacenter::Rack".into(),
            EntityDecl {
                name: "Rack".into(),
                fields: vec![
                    Field::Stored(StoredField {
                        name: "rack_id".into(),
                        ty: LuminaType::Text,
                        metadata: meta.clone(),
                        span,
                    }),
                    Field::Stored(StoredField {
                        name: "location".into(),
                        ty: LuminaType::Text,
                        metadata: meta.clone(),
                        span,
                    }),
                    Field::Stored(StoredField {
                        name: "total_capacity_u".into(),
                        ty: LuminaType::Number,
                        metadata: meta.clone(),
                        span,
                    }),
                    Field::Stored(StoredField {
                        name: "used_capacity_u".into(),
                        ty: LuminaType::Number,
                        metadata: meta.clone(),
                        span,
                    }),
                    Field::Stored(StoredField {
                        name: "total_power_kw".into(),
                        ty: LuminaType::Number,
                        metadata: meta.clone(),
                        span,
                    }),
                    Field::Stored(StoredField {
                        name: "ambient_temp_c".into(),
                        ty: LuminaType::Number,
                        metadata: meta.clone(),
                        span,
                    }),
                ],
                span,
            },
        );

        // LSL::datacenter::PDU
        self.schemas.insert(
            "LSL::datacenter::PDU".into(),
            EntityDecl {
                name: "PDU".into(),
                fields: vec![
                    Field::Stored(StoredField {
                        name: "pdu_id".into(),
                        ty: LuminaType::Text,
                        metadata: meta.clone(),
                        span,
                    }),
                    Field::Stored(StoredField {
                        name: "phase_a_amps".into(),
                        ty: LuminaType::Number,
                        metadata: meta.clone(),
                        span,
                    }),
                    Field::Stored(StoredField {
                        name: "phase_b_amps".into(),
                        ty: LuminaType::Number,
                        metadata: meta.clone(),
                        span,
                    }),
                    Field::Stored(StoredField {
                        name: "phase_c_amps".into(),
                        ty: LuminaType::Number,
                        metadata: meta.clone(),
                        span,
                    }),
                    Field::Stored(StoredField {
                        name: "total_power_kw".into(),
                        ty: LuminaType::Number,
                        metadata: meta.clone(),
                        span,
                    }),
                    Field::Stored(StoredField {
                        name: "voltage".into(),
                        ty: LuminaType::Number,
                        metadata: meta.clone(),
                        span,
                    }),
                ],
                span,
            },
        );

        // LSL::datacenter::CRAC
        self.schemas.insert(
            "LSL::datacenter::CRAC".into(),
            EntityDecl {
                name: "CRAC".into(),
                fields: vec![
                    Field::Stored(StoredField {
                        name: "unit_id".into(),
                        ty: LuminaType::Text,
                        metadata: meta.clone(),
                        span,
                    }),
                    Field::Stored(StoredField {
                        name: "supply_temp_c".into(),
                        ty: LuminaType::Number,
                        metadata: meta.clone(),
                        span,
                    }),
                    Field::Stored(StoredField {
                        name: "return_temp_c".into(),
                        ty: LuminaType::Number,
                        metadata: meta.clone(),
                        span,
                    }),
                    Field::Stored(StoredField {
                        name: "fan_speed_percent".into(),
                        ty: LuminaType::Number,
                        metadata: meta.clone(),
                        span,
                    }),
                    Field::Stored(StoredField {
                        name: "cooling_capacity_kw".into(),
                        ty: LuminaType::Number,
                        metadata: meta.clone(),
                        span,
                    }),
                    Field::Stored(StoredField {
                        name: "status".into(),
                        ty: LuminaType::Text,
                        metadata: meta.clone(),
                        span,
                    }),
                ],
                span,
            },
        );
    }

    // ── Network schemas ────────────────────────────────────

    fn register_network(&mut self) {
        let span = Span {
            start: 0,
            end: 0,
            line: 0,
            col: 0,
        };
        let meta = FieldMetadata {
            doc: None,
            range: None,
            affects: vec![],
        };

        // LSL::network::Switch
        self.schemas.insert(
            "LSL::network::Switch".into(),
            EntityDecl {
                name: "Switch".into(),
                fields: vec![
                    Field::Stored(StoredField {
                        name: "hostname".into(),
                        ty: LuminaType::Text,
                        metadata: meta.clone(),
                        span,
                    }),
                    Field::Stored(StoredField {
                        name: "mgmt_ip".into(),
                        ty: LuminaType::Text,
                        metadata: meta.clone(),
                        span,
                    }),
                    Field::Stored(StoredField {
                        name: "port_count".into(),
                        ty: LuminaType::Number,
                        metadata: meta.clone(),
                        span,
                    }),
                    Field::Stored(StoredField {
                        name: "uplink_gbps".into(),
                        ty: LuminaType::Number,
                        metadata: meta.clone(),
                        span,
                    }),
                    Field::Stored(StoredField {
                        name: "packet_loss_percent".into(),
                        ty: LuminaType::Number,
                        metadata: FieldMetadata {
                            doc: Some("Packet loss 0-100".into()),
                            range: Some((0.0, 100.0)),
                            affects: vec![],
                        },
                        span,
                    }),
                    Field::Stored(StoredField {
                        name: "cpu_percent".into(),
                        ty: LuminaType::Number,
                        metadata: meta.clone(),
                        span,
                    }),
                    Field::Stored(StoredField {
                        name: "status".into(),
                        ty: LuminaType::Text,
                        metadata: meta.clone(),
                        span,
                    }),
                ],
                span,
            },
        );

        // LSL::network::Router
        self.schemas.insert(
            "LSL::network::Router".into(),
            EntityDecl {
                name: "Router".into(),
                fields: vec![
                    Field::Stored(StoredField {
                        name: "hostname".into(),
                        ty: LuminaType::Text,
                        metadata: meta.clone(),
                        span,
                    }),
                    Field::Stored(StoredField {
                        name: "mgmt_ip".into(),
                        ty: LuminaType::Text,
                        metadata: meta.clone(),
                        span,
                    }),
                    Field::Stored(StoredField {
                        name: "bgp_peer_count".into(),
                        ty: LuminaType::Number,
                        metadata: meta.clone(),
                        span,
                    }),
                    Field::Stored(StoredField {
                        name: "route_table_size".into(),
                        ty: LuminaType::Number,
                        metadata: meta.clone(),
                        span,
                    }),
                    Field::Stored(StoredField {
                        name: "throughput_gbps".into(),
                        ty: LuminaType::Number,
                        metadata: meta.clone(),
                        span,
                    }),
                    Field::Stored(StoredField {
                        name: "status".into(),
                        ty: LuminaType::Text,
                        metadata: meta.clone(),
                        span,
                    }),
                ],
                span,
            },
        );

        // LSL::network::Firewall
        self.schemas.insert(
            "LSL::network::Firewall".into(),
            EntityDecl {
                name: "Firewall".into(),
                fields: vec![
                    Field::Stored(StoredField {
                        name: "hostname".into(),
                        ty: LuminaType::Text,
                        metadata: meta.clone(),
                        span,
                    }),
                    Field::Stored(StoredField {
                        name: "mgmt_ip".into(),
                        ty: LuminaType::Text,
                        metadata: meta.clone(),
                        span,
                    }),
                    Field::Stored(StoredField {
                        name: "active_sessions".into(),
                        ty: LuminaType::Number,
                        metadata: meta.clone(),
                        span,
                    }),
                    Field::Stored(StoredField {
                        name: "blocked_count".into(),
                        ty: LuminaType::Number,
                        metadata: meta.clone(),
                        span,
                    }),
                    Field::Stored(StoredField {
                        name: "throughput_mbps".into(),
                        ty: LuminaType::Number,
                        metadata: meta.clone(),
                        span,
                    }),
                    Field::Stored(StoredField {
                        name: "status".into(),
                        ty: LuminaType::Text,
                        metadata: meta.clone(),
                        span,
                    }),
                ],
                span,
            },
        );
    }

    // ── Power schemas ──────────────────────────────────────

    fn register_power(&mut self) {
        let span = Span {
            start: 0,
            end: 0,
            line: 0,
            col: 0,
        };
        let meta = FieldMetadata {
            doc: None,
            range: None,
            affects: vec![],
        };

        // LSL::power::UPS
        self.schemas.insert(
            "LSL::power::UPS".into(),
            EntityDecl {
                name: "UPS".into(),
                fields: vec![
                    Field::Stored(StoredField {
                        name: "unit_id".into(),
                        ty: LuminaType::Text,
                        metadata: meta.clone(),
                        span,
                    }),
                    Field::Stored(StoredField {
                        name: "battery_percent".into(),
                        ty: LuminaType::Number,
                        metadata: FieldMetadata {
                            doc: Some("Battery charge 0-100".into()),
                            range: Some((0.0, 100.0)),
                            affects: vec![],
                        },
                        span,
                    }),
                    Field::Stored(StoredField {
                        name: "load_percent".into(),
                        ty: LuminaType::Number,
                        metadata: meta.clone(),
                        span,
                    }),
                    Field::Stored(StoredField {
                        name: "input_voltage".into(),
                        ty: LuminaType::Number,
                        metadata: meta.clone(),
                        span,
                    }),
                    Field::Stored(StoredField {
                        name: "output_voltage".into(),
                        ty: LuminaType::Number,
                        metadata: meta.clone(),
                        span,
                    }),
                    Field::Stored(StoredField {
                        name: "runtime_minutes".into(),
                        ty: LuminaType::Number,
                        metadata: meta.clone(),
                        span,
                    }),
                    Field::Stored(StoredField {
                        name: "status".into(),
                        ty: LuminaType::Text,
                        metadata: meta.clone(),
                        span,
                    }),
                ],
                span,
            },
        );

        // LSL::power::Generator
        self.schemas.insert(
            "LSL::power::Generator".into(),
            EntityDecl {
                name: "Generator".into(),
                fields: vec![
                    Field::Stored(StoredField {
                        name: "unit_id".into(),
                        ty: LuminaType::Text,
                        metadata: meta.clone(),
                        span,
                    }),
                    Field::Stored(StoredField {
                        name: "fuel_percent".into(),
                        ty: LuminaType::Number,
                        metadata: meta.clone(),
                        span,
                    }),
                    Field::Stored(StoredField {
                        name: "output_kw".into(),
                        ty: LuminaType::Number,
                        metadata: meta.clone(),
                        span,
                    }),
                    Field::Stored(StoredField {
                        name: "frequency_hz".into(),
                        ty: LuminaType::Number,
                        metadata: meta.clone(),
                        span,
                    }),
                    Field::Stored(StoredField {
                        name: "runtime_hours".into(),
                        ty: LuminaType::Number,
                        metadata: meta.clone(),
                        span,
                    }),
                    Field::Stored(StoredField {
                        name: "status".into(),
                        ty: LuminaType::Text,
                        metadata: meta.clone(),
                        span,
                    }),
                ],
                span,
            },
        );
    }

    // ── Docker schemas ──────────────────────────────────────

    fn register_docker(&mut self) {
        let span = Span {
            start: 0,
            end: 0,
            line: 0,
            col: 0,
        };
        let meta = FieldMetadata {
            doc: None,
            range: None,
            affects: vec![],
        };

        // LSL::docker::Container
        self.schemas.insert(
            "LSL::docker::Container".into(),
            EntityDecl {
                name: "Container".into(),
                fields: vec![
                    Field::Stored(StoredField {
                        name: "name".into(),
                        ty: LuminaType::Text,
                        metadata: meta.clone(),
                        span,
                    }),
                    Field::Stored(StoredField {
                        name: "image".into(),
                        ty: LuminaType::Text,
                        metadata: meta.clone(),
                        span,
                    }),
                    Field::Stored(StoredField {
                        name: "port".into(),
                        ty: LuminaType::Number,
                        metadata: meta.clone(),
                        span,
                    }),
                    Field::Stored(StoredField {
                        name: "target_port".into(),
                        ty: LuminaType::Number,
                        metadata: meta.clone(),
                        span,
                    }),
                    Field::Stored(StoredField {
                        name: "env_vars".into(),
                        ty: LuminaType::Text,
                        metadata: meta.clone(),
                        span,
                    }),
                    Field::Stored(StoredField {
                        name: "status".into(),
                        ty: LuminaType::Text,
                        metadata: meta.clone(),
                        span,
                    }),
                    Field::Stored(StoredField {
                        name: "verified".into(),
                        ty: LuminaType::Boolean,
                        metadata: meta.clone(),
                        span,
                    }),
                    Field::Stored(StoredField {
                        name: "tier".into(),
                        ty: LuminaType::Text,
                        metadata: meta.clone(),
                        span,
                    }),
                ],
                span,
            },
        );
    }
}
