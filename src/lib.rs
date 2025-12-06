// Web assembly
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
//use web_sys::{Response, window};
use serde_json::json;
use liquid::model::Value; // Needed for translation
use liquid::model::Object; // Needed for translation

mod filecontent;
mod query;
mod render;

const TEMPLATE_MENU: &str = include_str!("../templates/_menu.liquid");
const TEMPLATE_OVERVIEW: &str = include_str!("../templates/overview.liquid");
const QUERY_OVERVIEW_YEAR: &str = include_str!("../queries/overview_year.sql");
const QUERY_OVERVIEW_COUNTRY: &str = include_str!("../queries/overview_country.sql");

#[wasm_bindgen(start)]
fn start() {

    wasm_bindgen_futures::spawn_local(async {

        // -----------------------------------------------------------------------
        // First: Get sqlite database binary
        // -----------------------------------------------------------------------

        let db_bytes = filecontent::get_sqlite_binary().await;
        // You can now use db_bytes, e.g. log length
        web_sys::console::log_1(&format!("DB size: {}", db_bytes.len()).into());

        // IF NOT db_bytes then go to ?p=configure


        // -----------------------------------------------------------------------
        // Second: Run queries in sqlite database
        // -----------------------------------------------------------------------

        use serde_json::Value as JsonValue;

        // Run the query
        let translation_query = vec![
            ("settings".to_string(), "SELECT Value FROM bewxx_Settings WHERE AttributeGroup = 'Base' AND Attribute = 'LanguageFile';".to_string())
        ];
        let translation_filename = query::get_query_data(&db_bytes, translation_query).await;

        let json_obj: JsonValue = serde_json::to_value(&translation_filename).unwrap();

        let translation_filename_extracted = format!(
            "languages/{}",
            json_obj["settings"][0]["Value"]
            .as_str()
            .expect("Expected settings[0].Value to be a string")
        );

        web_sys::console::log_1(&translation_filename_extracted.as_str().into());

        let translation_content = filecontent::fetch_json(&translation_filename_extracted).await;

        web_sys::console::log_1(
            &serde_json::to_string(&translation_content).unwrap().into()
        );



        let queries = vec![
            ("overviewYear".to_string(), QUERY_OVERVIEW_YEAR.to_string()),
            ("overviewCountry".to_string(), QUERY_OVERVIEW_COUNTRY.to_string()),
            ("settings".to_string(), "SELECT * FROM bewxx_Settings;".to_string()),
            ("tripDomains".to_string(), "SELECT * FROM bewx_TripDomains WHERE DomainAbbreviation != 'X';".to_string()),
            ("participantGroups".to_string(), "SELECT * FROM bewx_ParticipantGroups;".to_string())
        ];
        let mut liquid_obj = query::get_query_data(&db_bytes, queries).await;


        use liquid::Object;

        match translation_content.as_ref() {
            Some(content) => {
                let translation_obj = liquid::model::to_object(content)
                .unwrap_or_else(|_| liquid::Object::new());
                liquid_obj.insert("translation".into(), liquid::model::Value::Object(translation_obj));
            }
            None => {
                liquid_obj.insert("translation".into(), liquid::model::Value::Nil);
            }
        }


        web_sys::console::log_1(
            &serde_json::to_string(&liquid_obj).unwrap().into()
        );

        // -----------------------------------------------------------------------
        // Third: Render to html
        // -----------------------------------------------------------------------

        render::render_to_dom(TEMPLATE_OVERVIEW, &liquid_obj, "app");


    });

}
