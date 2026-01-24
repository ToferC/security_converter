use async_graphql::{Error};
use diesel::{self, RunQueryDsl};
use rand::Rng;
use rand::seq::SliceRandom;

use crate::models::{
    Authority, ConversionRequest, DataObject, InsertableConversionRequest, InsertableDataObject,
    InsertableMetadata, Metadata, Nation, NewAuthority, NewClassificationSchema,
    NewNation, User,
};
use crate::progress::progress::ProgressLogger;
use crate::{database, schema::*};

/// Creates basic test objects in the database for testing
pub fn pre_populate_db_schema() -> Result<(), Error> {
    // Get all users to assign as creators
    let users = User::get_all()?;
    if users.is_empty() {
        return Err(Error::new(
            "No users found in database. Please create users first.",
        ));
    }

    let mut rng = rand::thread_rng();

    // ========== Create NATO Nations ==========
    println!("\n========== Creating NATO Nations ==========");
    let nations_data = vec![
        ("USA", "United States"),
        ("GBR", "United Kingdom"),
        ("FRA", "France"),
        ("DEU", "Germany"),
        ("ITA", "Italy"),
        ("CAN", "Canada"),
        ("ESP", "Spain"),
        ("NLD", "Netherlands"),
        ("POL", "Poland"),
        ("TUR", "Turkey"),
        ("NOR", "Norway"),
        ("BEL", "Belgium"),
        ("DNK", "Denmark"),
        ("PRT", "Portugal"),
        ("CZE", "Czech Republic"),
        ("HUN", "Hungary"),
        ("GRC", "Greece"),
        ("ROU", "Romania"),
        ("BGR", "Bulgaria"),
        ("SVK", "Slovakia"),
        ("ISL", "Iceland"),
        ("LUX", "Luxembourg"),
        ("EST", "Estonia"),
        ("LVA", "Latvia"),
        ("LTU", "Lithuania"),
        ("SVN", "Slovenia"),
        ("ALB", "Albania"),
        ("HRV", "Croatia"),
        ("MNE", "Montenegro"),
        ("MKD", "North Macedonia"),
        ("FIN", "Finland"),
        ("SWE", "Sweden"),
    ];

    let mut new_nations = Vec::new();
    for (code, name) in nations_data.iter() {
        let creator = users.choose(&mut rng).unwrap();
        new_nations.push(NewNation::new(
            creator.id,
            code.to_string(),
            name.to_string(),
        ));
    }

    let mut progress_nations =
        ProgressLogger::new("Inserting Nations".to_owned(), new_nations.len());

    // Batch insert nations
    let mut conn = database::connection()?;
    let inserted_count = diesel::insert_into(nations::table)
        .values(&new_nations)
        .execute(&mut conn)?;

    progress_nations.done();
    println!("✓ Inserted {} nations", inserted_count);

    // Retrieve created nations for foreign key references
    let created_nations = Nation::get_all()?;

    // ========== Create Authorities ==========
    println!("\n========== Creating Authorities ==========");
    let authority_titles = vec![
        "National Security Authority",
        "Defense Intelligence Agency",
        "Ministry of Defence Security",
        "National Classification Office",
        "Military Intelligence Directorate",
        "Security Clearance Division",
    ];

    let mut new_authorities = Vec::new();

    for nation in created_nations.iter() {
        let num_authorities = rng.gen_range(1..=3);

        for i in 0..num_authorities {
            let creator = users.choose(&mut rng).unwrap();
            let title = authority_titles.choose(&mut rng).unwrap();

            let name = format!("{} - {}", nation.nation_name, title);
            let email = format!(
                "{}{}@{}.nato.int",
                title.to_lowercase().replace(" ", "."),
                if i > 0 {
                    format!("{}", i)
                } else {
                    String::new()
                },
                nation.nation_code.to_lowercase()
            );
            let phone = format!(
                "+{}-{}-{}-{}",
                rng.gen_range(1..999),
                rng.gen_range(100..999),
                rng.gen_range(100..999),
                rng.gen_range(1000..9999)
            );

            // 30% chance to have expiration date
            let expires_at = if rng.gen_bool(0.3) {
                Some(
                    chrono::Utc::now().naive_utc()
                        + chrono::Duration::days(rng.gen_range(365..1095)),
                )
            } else {
                None
            };

            new_authorities.push(NewAuthority::new(
                creator.id, nation.id, name, email, phone, expires_at,
            ));
        }
    }

    let mut progress_authorities =
        ProgressLogger::new("Inserting Authorities".to_owned(), new_authorities.len());

    // Batch insert authorities
    let inserted_authorities = diesel::insert_into(authorities::table)
        .values(&new_authorities)
        .execute(&mut conn)?;

    progress_authorities.done();
    println!("✓ Inserted {} authorities", inserted_authorities);

    // Retrieve created authorities for foreign key references
    let created_authorities = Authority::get_all()?;

    // ========== Create Classification Schemas ==========
    println!("\n========== Creating Classification Schemas ==========");

    // Realistic classification mappings for various NATO nations
    let classification_mappings = vec![
        (
            "USA",
            vec![
                (
                    "UNCLASSIFIED",
                    "CONFIDENTIAL",
                    "CONFIDENTIAL",
                    "SECRET",
                    "TOP SECRET",
                ),
                (
                    "UNCLASSIFIED",
                    "CONFIDENTIAL",
                    "CONFIDENTIAL",
                    "SECRET",
                    "TOP SECRET",
                ),
            ],
        ),
        (
            "GBR",
            vec![
                (
                    "OFFICIAL",
                    "OFFICIAL-SENSITIVE",
                    "SECRET",
                    "SECRET",
                    "TOP SECRET",
                ),
                (
                    "OFFICIAL",
                    "OFFICIAL-SENSITIVE",
                    "SECRET",
                    "SECRET",
                    "TOP SECRET",
                ),
            ],
        ),
        (
            "FRA",
            vec![
                (
                    "Non Protégé",
                    "Diffusion Restreinte",
                    "Confidentiel Défense",
                    "Secret Défense",
                    "Très Secret Défense",
                ),
                (
                    "Non Protégé",
                    "Diffusion Restreinte",
                    "Confidentiel Défense",
                    "Secret Défense",
                    "Très Secret Défense",
                ),
            ],
        ),
        (
            "DEU",
            vec![
                (
                    "Offen",
                    "VS-NfD",
                    "VS-Vertraulich",
                    "Geheim",
                    "Streng Geheim",
                ),
                (
                    "Offen",
                    "VS-NfD",
                    "VS-Vertraulich",
                    "Geheim",
                    "Streng Geheim",
                ),
            ],
        ),
        (
            "ITA",
            vec![
                (
                    "Non Classificato",
                    "Riservato",
                    "Riservatissimo",
                    "Segreto",
                    "Segretissimo",
                ),
                (
                    "Non Classificato",
                    "Riservato",
                    "Riservatissimo",
                    "Segreto",
                    "Segretissimo",
                ),
            ],
        ),
        (
            "CAN",
            vec![
                (
                    "UNCLASSIFIED",
                    "PROTECTED A",
                    "PROTECTED B",
                    "SECRET",
                    "TOP SECRET",
                ),
                (
                    "UNCLASSIFIED",
                    "PROTECTED A",
                    "PROTECTED B",
                    "SECRET",
                    "TOP SECRET",
                ),
            ],
        ),
        (
            "ESP",
            vec![
                (
                    "No Clasificado",
                    "Difusión Limitada",
                    "Confidencial",
                    "Secreto",
                    "Alto Secreto",
                ),
                (
                    "No Clasificado",
                    "Difusión Limitada",
                    "Confidencial",
                    "Secreto",
                    "Alto Secreto",
                ),
            ],
        ),
        (
            "NLD",
            vec![
                (
                    "Niet-Gerubriceerd",
                    "Departementaal Vertrouwelijk",
                    "Confidentieel",
                    "Geheim",
                    "Zeer Geheim",
                ),
                (
                    "Niet-Gerubriceerd",
                    "Departementaal Vertrouwelijk",
                    "Confidentieel",
                    "Geheim",
                    "Zeer Geheim",
                ),
            ],
        ),
        (
            "POL",
            vec![
                ("Jawne", "Zastrzeżone", "Poufne", "Tajne", "Ściśle Tajne"),
                ("Jawne", "Zastrzeżone", "Poufne", "Tajne", "Ściśle Tajne"),
            ],
        ),
        (
            "TUR",
            vec![
                ("Açık", "Hizmete Özel", "Özel", "Gizli", "Çok Gizli"),
                ("Açık", "Hizmete Özel", "Özel", "Gizli", "Çok Gizli"),
            ],
        ),
        (
            "EST",
            vec![
                ("Avalik", "Piiratud", "Konfidentsiaalne", "Salajane", "Täiesti Salajane"),
                ("Avalik", "Piiratud", "Konfidentsiaalne", "Salajane", "Täiesti Salajane"),
            ],
        ),
        (
            "HRV",
            vec![
                ("Javno", "Ograničeno", "Povjerljivo", "Tajno", "Vrlo Tajno"),
                ("Javno", "Ograničeno", "Povjerljivo", "Tajno", "Vrlo Tajno"),
            ],
        ),
        (
            "SVN",
            vec![
                ("Javno", "Omejeno", "Zaupno", "Tajno", "Strogo Tajno"),
                ("Javno", "Omejeno", "Zaupno", "Tajno", "Strogo Tajno"),
            ],
        ),
        (
            "FIN",
            vec![
                ("Julkinen", "Käyttö Rajoitettu", "Luottamuksellinen", "Salainen", "Erittäin Salainen"),
                ("Julkinen", "Käyttö Rajoitettu", "Luottamuksellinen", "Salainen", "Erittäin Salainen"),
            ],
        ),
        (
            "SWE",
            vec![
                ("Öppen", "Begränsad", "Konfidentiell", "Hemlig", "Kvalificerat Hemlig"),
                ("Öppen", "Begränsad", "Konfidentiell", "Hemlig", "Kvalificerat Hemlig"),
            ],
        ),
        (
            "LVA",
            vec![
                ("Atklāts", "Ierobežotas", "Konfidenciāli", "Slepeni", "Sevišķi Slepeni"),
                ("Atklāts", "Ierobežotas", "Konfidenciāli", "Slepeni", "Sevišķi Slepeni"),
            ],
        ),
        (
            "LTU",
            vec![
                ("Viešas", "Riboto Naudojimo", "Konfidencialus", "Slaptai", "Visiškai Slaptai"),
                ("Viešas", "Riboto Naudojimo", "Konfidencialus", "Slaptai", "Visiškai Slaptai"),
            ],
        ),
    ];

    let caveats_options = vec![
        "NOFORN",
        "REL TO NATO",
        "EYES ONLY",
        "ORIGINATOR CONTROLLED",
        "RELEASABLE TO",
        "NATO UNCLASSIFIED",
        "",
    ];

    let mut new_classification_schemas = Vec::new();

    for nation in created_nations.iter() {
        // Find matching classification mapping
        if let Some(mapping) = classification_mappings
            .iter()
            .find(|(code, _)| code == &nation.nation_code.as_str())
        {

            // Create 1-2 versions for this nation
            let num_versions = if rng.gen_bool(0.3) { 2 } else { 1 };

            for version_num in 1..=num_versions {
                let creator = users.choose(&mut rng).unwrap();
                let caveat = caveats_options.choose(&mut rng).unwrap();

                // Use the appropriate mapping tuple (all same for realistic data)
                let levels = mapping.1[0];

                // 20% chance to have expiration date
                let expires_at = if rng.gen_bool(0.2) {
                    Some(
                        chrono::Utc::now().naive_utc()
                            + chrono::Duration::days(rng.gen_range(730..1825)), // 2-5 years
                    )
                } else {
                    None
                };

                new_classification_schemas.push(NewClassificationSchema::new(
                    creator.id,
                    nation.nation_code.clone(),
                    // TO NATO mappings
                    levels.0.to_string(),
                    levels.1.to_string(),
                    levels.2.to_string(),
                    levels.3.to_string(),
                    levels.4.to_string(),
                    // FROM NATO mappings (same as TO for simplicity)
                    levels.0.to_string(),
                    levels.1.to_string(),
                    levels.2.to_string(),
                    levels.3.to_string(),
                    levels.4.to_string(),
                    caveat.to_string(),
                    format!("v{}.0", version_num),
                    nation.id,
                    expires_at,
                ));
            }
        } else {
            // For nations without specific mappings, create a generic one
            let creator = users.choose(&mut rng).unwrap();

                let caveat = caveats_options.choose(&mut rng).unwrap();

                let expires_at = if rng.gen_bool(0.2) {
                    Some(
                        chrono::Utc::now().naive_utc()
                            + chrono::Duration::days(rng.gen_range(730..1825)),
                    )
                } else {
                    None
                };

                new_classification_schemas.push(NewClassificationSchema::new(
                    creator.id,
                    nation.nation_code.clone(),
                    "UNCLASSIFIED".to_string(),
                    "RESTRICTED".to_string(),
                    "CONFIDENTIAL".to_string(),
                    "SECRET".to_string(),
                    "TOP SECRET".to_string(),
                    "UNCLASSIFIED".to_string(),
                    "RESTRICTED".to_string(),
                    "CONFIDENTIAL".to_string(),
                    "SECRET".to_string(),
                    "TOP SECRET".to_string(),
                    caveat.to_string(),
                    "v1.0".to_string(),
                    nation.id,
                    expires_at,
                ));
            }
        }


    let mut progress_schemas = ProgressLogger::new(
        "Inserting Classification Schemas".to_owned(),
        new_classification_schemas.len(),
    );

    // Batch insert classification schemas
    let inserted_schemas = diesel::insert_into(classification_schemas::table)
        .values(&new_classification_schemas)
        .execute(&mut conn)?;

    progress_schemas.done();
    println!("✓ Inserted {} classification schemas", inserted_schemas);

    // ========== Create Metadata ==========
    println!("\n========== Creating Metadata ==========");
    let metadata_domains = vec![
        (
            "INTEL",
            vec![
                "sigint",
                "humint",
                "geoint",
                "osint",
                "classified",
                "top-secret",
            ],
        ),
        (
            "CYBER",
            vec![
                "cyber-defense",
                "network-security",
                "encryption",
                "firewall",
                "threat-intelligence",
            ],
        ),
        (
            "OPERATIONS",
            vec![
                "tactical",
                "strategic",
                "mission-critical",
                "operational-security",
                "field-ops",
            ],
        ),
        (
            "LOGISTICS",
            vec![
                "supply-chain",
                "transportation",
                "equipment",
                "maintenance",
                "procurement",
            ],
        ),
        (
            "COMMUNICATIONS",
            vec!["comms", "radio", "satellite", "secure-voice", "data-link"],
        ),
        (
            "NUCLEAR",
            vec![
                "nuclear-security",
                "wmd",
                "nonproliferation",
                "arms-control",
                "strategic-weapons",
            ],
        ),
        (
            "COUNTERTERRORISM",
            vec![
                "ct-ops",
                "threat-assessment",
                "surveillance",
                "counterintelligence",
                "security-clearance",
            ],
        ),
        (
            "MARITIME",
            vec![
                "naval-ops",
                "submarine",
                "surface-warfare",
                "maritime-patrol",
                "anti-submarine",
            ],
        ),
        (
            "AEROSPACE",
            vec![
                "air-defense",
                "fighter-ops",
                "reconnaissance",
                "space-operations",
                "satellite-imagery",
            ],
        ),
        (
            "SPECIAL-OPS",
            vec![
                "special-forces",
                "covert-ops",
                "direct-action",
                "reconnaissance",
                "hostage-rescue",
            ],
        ),
    ];

    // Note: Data Objects and Metadata are now created via ConversionRequests
    // This ensures all data objects originate from actual conversion workflows

    // ========== Create Conversion Requests ==========
    println!("\n========== Creating Conversion Requests (with Data Objects & Metadata) ==========");

    let mut created_conversion_requests = 0;
    let mut conversion_errors = 0;

    // Templates for conversion request data objects
    let conversion_titles = vec![
        "Intelligence Report",
        "Tactical Assessment",
        "Strategic Document",
        "Operational Brief",
        "Security Analysis",
        "Mission Data",
        "Threat Intelligence",
        "Classified Communication",
        "Defense Protocol",
        "Military Exercise Report",
        "Operational Report",
        "Intelligence Assessment",
        "Classified Briefing",
        "Mission Planning Document",
        "Security Protocol",
        "Threat Analysis",
        "Equipment Specifications",
        "Communications Plan",
        "Training Manual",
        "Strategic Assessment",
        "Incident Report",
        "Intelligence Summary",
        "Operational Order",
        "Capability Analysis",
        "Security Clearance Review",
        "After-Action Report",
        "Tactical Intelligence Brief",
        "Joint Operations Plan",
        "Force Protection Assessment",
        "Signals Intelligence Report",
    ];

    let conversion_domains = vec![
        "INTEL", "CYBER", "OPERATIONS", "LOGISTICS", "COMMUNICATIONS",
        "NUCLEAR", "COUNTERTERRORISM", "MARITIME", "AEROSPACE", "SPECIAL-OPS",
    ];

    for authority in &created_authorities {
        // Create random number of conversion requests

        for i in 1..rng.gen_range(40..=50) {

            let creator = users.choose(&mut rng).unwrap();
            let source_nation = created_nations.iter()
                .find(|&n| n.id == authority.nation_id)
                .unwrap();

            // Select 1-3 random target nations (different from source)
            let num_targets = rng.gen_range(1..=8);
            let mut target_nations: Vec<String> = created_nations
                .iter()
                .filter(|n| n.nation_code != source_nation.nation_code)
                .collect::<Vec<_>>()
                .choose_multiple(&mut rng, num_targets)
                .map(|n| n.nation_code.clone())
                .collect();

            // Ensure at least one target nation
            if target_nations.is_empty() {
                target_nations.push(
                    created_nations
                        .iter()
                        .find(|n| n.nation_code != source_nation.nation_code)
                        .unwrap_or(&source_nation)
                        .nation_code
                        .clone()
                );
            }

            // Create insertable data object for the conversion request
            let title_template = conversion_titles.choose(&mut rng).unwrap();
            let title = format!("{} - {} #{}", title_template, source_nation.nation_code, rng.gen_range(1000..9999));
            let description = format!(
                "Conversion request for {} from {} to {:?}. Created for testing purposes.",
                title_template.to_lowercase(),
                source_nation.nation_name,
                target_nations
            );

            let insertable_data_object = InsertableDataObject {
                title: title.clone(),
                description: description.clone(),
            };

            // Create insertable metadata
            let domain = conversion_domains.choose(&mut rng).unwrap();

            let default_tags = &vec!["general", "nato", "classified"];
            
            let tags = metadata_domains
                .iter()
                .find(|(d, _)| d == domain)
                .map(|(_, tags)| tags)
                .unwrap_or(default_tags);

            let num_tags = rng.gen_range(1..=3);

            let selected_tags: Vec<Option<String>> = tags
                .choose_multiple(&mut rng, num_tags)
                .map(|s| Some(s.to_string()))
                .collect();

            // Get the classification schema for the source nation to use correct terminology
            let source_classification = match crate::models::ClassificationSchema::get_latest_by_nation_code(
                &source_nation.nation_code
            ) {
                Ok(schema) => {
                    // Pick a random classification level from the source nation's schema
                    let classifications = vec![
                        &schema.to_nato_unclassified,
                        &schema.to_nato_restricted,
                        &schema.to_nato_confidential,
                        &schema.to_nato_secret,
                        &schema.to_nato_top_secret,
                    ];
                    classifications.choose(&mut rng).unwrap().to_string()
                }
                Err(_) => {
                    // Fallback to generic NATO classifications if schema not found
                    eprintln!("Warning: No classification schema found for nation {}. Skipping request.", source_nation.nation_code);
                    conversion_errors += 1;
                    continue; // Skip this request
                }
            };

            // Generate realistic metadata with proper security classification fields
            // Use authorities from the SOURCE nation for metadata
            let source_nation_authorities: Vec<&Authority> = created_authorities
                .iter()
                .filter(|a| {
                    // Find the nation for this authority
                    created_nations.iter().any(|n| n.id == a.nation_id && n.nation_code == source_nation.nation_code)
                })
                .collect();

            // If no authorities found for source nation, skip this request
            if source_nation_authorities.is_empty() {
                eprintln!("Warning: No authorities found for source nation {}. Skipping request.", source_nation.nation_code);
                conversion_errors += 1;
                continue;
            }

            let originator_org = source_nation_authorities.choose(&mut rng).unwrap();
            let custodian_org = source_nation_authorities.choose(&mut rng).unwrap();

            // Format types
            let formats = vec!["application/pdf", "text/plain", "application/json", "image/jpeg", "application/xml", "application/msword"];
            let format = formats.choose(&mut rng).unwrap().to_string();
            let format_size = if rng.gen_bool(0.8) { Some(rng.gen_range(1024..10485760)) } else { None };

            // Releasability - include source nation and possibly target nations
            let releasable_to_countries = if rng.gen_bool(0.7) {
                let mut countries = vec![Some(source_nation.nation_code.clone())];
                // 50% chance to include target nations
                if rng.gen_bool(0.5) {
                    for target in &target_nations {
                        countries.push(Some(target.clone()));
                    }
                }
                Some(countries)
            } else {
                None
            };

            let releasable_to_organizations = if rng.gen_bool(0.4) {
                Some(vec![Some("NATO".to_string()), Some("FVEY".to_string())])
            } else {
                None
            };

            let releasable_to_categories = if rng.gen_bool(0.3) {
                Some(vec![Some("military".to_string()), Some("intelligence".to_string())])
            } else {
                None
            };

            let disclosure_category = if rng.gen_bool(0.5) {
                let categories = vec!["Category A", "Category B", "Category C"];
                Some(categories.choose(&mut rng).unwrap().to_string())
            } else {
                None
            };

            // Handling restrictions
            let handling_restrictions = if rng.gen_bool(0.6) {
                let restrictions = vec!["NOFORN", "ORCON", "PROPIN", "REL TO", "EYES ONLY"];
                let num_restrictions = rng.gen_range(1..=3);
                Some(
                    restrictions
                        .choose_multiple(&mut rng, num_restrictions)
                        .map(|s| Some(s.to_string()))
                        .collect()
                )
            } else {
                None
            };

            let handling_authority = if handling_restrictions.is_some() {
                let authorities = vec![
                    "DoD Directive 5230.09",
                    "NATO STANAG 2161",
                    "Executive Order 13526",
                    "National Security Act",
                ];
                Some(authorities.choose(&mut rng).unwrap().to_string())
            } else {
                None
            };

            let no_handling_restrictions = if handling_restrictions.is_none() {
                Some(true)
            } else {
                None
            };

            // Authorization reference
            let authorization_refs = vec![
                "Executive Order 13526",
                "DoD 5220.22-M NISPOM",
                "NATO STANAG 2161",
                "Court Order 2024-123",
                "MOU-NATO-2023",
                "OPORD 24-001",
                "FRAGO 12-2024",
            ];
            let authorization_reference = if rng.gen_bool(0.7) {
                Some(authorization_refs.choose(&mut rng).unwrap().to_string())
            } else {
                None
            };

            let authorization_reference_date = if authorization_reference.is_some() {
                Some(
                    chrono::Utc::now().naive_utc() - chrono::Duration::days(rng.gen_range(1..365))
                )
            } else {
                None
            };

            let insertable_metadata = InsertableMetadata {
                identifier: format!("DOC-{}-{}-{}",
                    source_nation.nation_code,
                    domain,
                    uuid::Uuid::new_v4().to_string().split('-').next().unwrap()
                ),
                authorization_reference,
                authorization_reference_date,
                originator_organization_id: originator_org.id,
                custodian_organization_id: custodian_org.id,
                format,
                format_size,
                security_classification: source_classification.clone(),
                releasable_to_countries,
                releasable_to_organizations,
                releasable_to_categories,
                disclosure_category,
                handling_restrictions,
                handling_authority,
                no_handling_restrictions,
                domain: domain.to_string(),
                tags: selected_tags,
            };

            let conversion_payload = InsertableConversionRequest {
                user_id: creator.id,
                authority_id: authority.id,
                data_object: insertable_data_object,
                metadata: insertable_metadata,
                source_nation_classification: source_classification,
                source_nation_code: source_nation.nation_code.clone(),
                target_nation_codes: target_nations,
            };

            // Process the payload to create the conversion request and then convert it
            match ConversionRequest::process_payload(&conversion_payload) {
                Ok(mut request) => {
                    // Now process the conversion to generate the response
                    match request.process_and_convert() {
                        Ok(_response) => {
                            created_conversion_requests += 1;
                        }
                        Err(e) => {
                            eprintln!("Failed to convert request {}: {:?}", i + 1, e);
                            // Still count the request as created even if conversion fails
                            created_conversion_requests += 1;
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Failed to create conversion request {}: {:?}", i + 1, e);
                    conversion_errors += 1;
                }
            }
        }
    }

    let mut progress_conversion_requests =
        ProgressLogger::new("Inserting Conversion Requests".to_owned(), created_conversion_requests as usize);
    progress_conversion_requests.done();
    println!("✓ Inserted {} conversion requests ({} errors)", created_conversion_requests, conversion_errors);

    // Count the created data objects and metadata
    let total_data_objects = DataObject::get_all()?.len();
    let total_metadata = Metadata::get_all()?.len();

    // ========== Summary ==========
    println!("\n========================================");
    println!("✓ Database population complete!");
    println!("========================================");
    println!("  • {} Nations", inserted_count);
    println!("  • {} Authorities", inserted_authorities);
    println!("  • {} Classification Schemas", inserted_schemas);
    println!("  • {} Conversion Requests", created_conversion_requests);
    println!("  • {} DataObjects (created via requests)", total_data_objects);
    println!("  • {} Metadata records (created via requests)", total_metadata);
    if conversion_errors > 0 {
        println!("  ⚠ {} conversion errors", conversion_errors);
    }
    println!("========================================\n");

    Ok(())
}

pub fn gen_rand_number() -> String {
    let mut rng = rand::thread_rng();

    let rand_num: String = (0..11)
        .map(|_| {
            let i = rng.gen_range(0..10);
            i.to_string()
        })
        .collect();

    rand_num
}
