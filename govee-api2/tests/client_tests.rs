//! Integration tests exercising every public client method against a mock
//! HTTP server. Fixture JSON is modeled on the examples in Govee's platform
//! API reference (developer.govee.com/reference).

use std::time::Duration;

use govee_api2::{ClientConfig, Color, Error, GoveeClient, Scene};
use serde_json::json;
use wiremock::matchers::{body_partial_json, header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

const DEVICE: &str = "8C:2E:9C:04:A0:03:82:D1";
const SKU: &str = "H618E";

async fn client_for(server: &MockServer) -> GoveeClient {
    GoveeClient::with_config(
        "test-api-key",
        ClientConfig {
            timeout: Duration::from_secs(5),
            retry_attempts: 3,
            base_url: server.uri(),
        },
    )
}

async fn client_without_retries(server: &MockServer) -> GoveeClient {
    GoveeClient::with_config(
        "test-api-key",
        ClientConfig {
            timeout: Duration::from_secs(5),
            retry_attempts: 0,
            base_url: server.uri(),
        },
    )
}

fn success_control_response() -> ResponseTemplate {
    ResponseTemplate::new(200).set_body_json(json!({
        "requestId": "uuid",
        "msg": "success",
        "code": 200,
        "capability": {}
    }))
}

// --- get_devices -----------------------------------------------------------

#[tokio::test]
async fn get_devices_happy_path() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/router/api/v1/user/devices"))
        .and(header("Govee-API-Key", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "code": 200,
            "message": "success",
            "data": [
                {
                    "sku": "H6072",
                    "device": "9D:FA:85:EB:D3:00:8B:FF",
                    "deviceName": "Floor Lamp",
                    "type": "devices.types.light",
                    "capabilities": [
                        { "type": "devices.capabilities.on_off", "instance": "powerSwitch",
                          "parameters": { "dataType": "ENUM" } },
                        { "type": "devices.capabilities.range", "instance": "brightness",
                          "parameters": { "dataType": "INTEGER" } },
                        { "type": "devices.capabilities.color_setting", "instance": "colorRgb",
                          "parameters": { "dataType": "INTEGER" } },
                        { "type": "devices.capabilities.color_setting", "instance": "colorTemperatureK",
                          "parameters": { "dataType": "INTEGER" } },
                        { "type": "devices.capabilities.dynamic_scene", "instance": "lightScene",
                          "parameters": { "dataType": "ENUM" } },
                        { "type": "devices.capabilities.segment_color_setting",
                          "instance": "segmentedColorRgb",
                          "parameters": { "dataType": "STRUCT" } }
                    ]
                },
                {
                    "sku": "SameModeGroup",
                    "device": "group-1",
                    "deviceName": "Living Room",
                    "type": null
                }
            ]
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = client_for(&server).await;
    let devices = client.get_devices().await.unwrap();

    assert_eq!(devices.len(), 2);
    let lamp = &devices[0];
    assert_eq!(lamp.device_name, "Floor Lamp");
    assert!(lamp.supports_power());
    assert!(lamp.supports_brightness());
    assert!(lamp.supports_color());
    assert!(lamp.supports_color_temp());
    assert!(lamp.supports_scenes());
    assert!(lamp.supports_segments());
    assert!(!lamp.is_group());

    let group = &devices[1];
    assert!(group.is_group());
    assert!(group.capabilities.is_empty());
}

// --- get_device_state ------------------------------------------------------

#[tokio::test]
async fn get_device_state_happy_path() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/router/api/v1/device/state"))
        .and(header("Govee-API-Key", "test-api-key"))
        .and(body_partial_json(
            json!({ "payload": { "sku": SKU, "device": DEVICE } }),
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "requestId": "uuid",
            "msg": "success",
            "code": 200,
            "payload": {
                "sku": SKU,
                "device": DEVICE,
                "capabilities": [
                    { "type": "devices.capabilities.online", "instance": "online",
                      "state": { "value": true } },
                    { "type": "devices.capabilities.on_off", "instance": "powerSwitch",
                      "state": { "value": 1 } },
                    { "type": "devices.capabilities.range", "instance": "brightness",
                      "state": { "value": 75 } },
                    { "type": "devices.capabilities.color_setting", "instance": "colorRgb",
                      "state": { "value": 255 } },
                    { "type": "devices.capabilities.color_setting", "instance": "colorTemperatureK",
                      "state": { "value": 0 } },
                    { "type": "devices.capabilities.dynamic_scene", "instance": "lightScene",
                      "state": { "value": 3853 } }
                ]
            }
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = client_for(&server).await;
    let state = client.get_device_state(DEVICE, SKU).await.unwrap();

    assert!(state.power);
    assert_eq!(state.online, Some(true));
    assert_eq!(state.brightness, Some(75));
    let color = state.color.unwrap();
    assert_eq!((color.r, color.g, color.b), (0, 0, 255));
    assert_eq!(state.color_temperature_kelvin, Some(0));
    assert_eq!(state.light_scene, Some(3853));
}

// --- power / brightness / color / temperature control ----------------------

#[tokio::test]
async fn turn_on_sends_power_switch_capability() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/router/api/v1/device/control"))
        .and(body_partial_json(json!({
            "payload": {
                "sku": SKU,
                "device": DEVICE,
                "capability": {
                    "type": "devices.capabilities.on_off",
                    "instance": "powerSwitch",
                    "value": 1
                }
            }
        })))
        .respond_with(success_control_response())
        .expect(1)
        .mount(&server)
        .await;

    let client = client_for(&server).await;
    client.turn_on(DEVICE, SKU).await.unwrap();
}

#[tokio::test]
async fn turn_off_sends_power_switch_capability() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/router/api/v1/device/control"))
        .and(body_partial_json(json!({
            "payload": {
                "capability": {
                    "type": "devices.capabilities.on_off",
                    "instance": "powerSwitch",
                    "value": 0
                }
            }
        })))
        .respond_with(success_control_response())
        .expect(1)
        .mount(&server)
        .await;

    let client = client_for(&server).await;
    client.turn_off(DEVICE, SKU).await.unwrap();
}

#[tokio::test]
async fn set_brightness_clamps_to_100() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/router/api/v1/device/control"))
        .and(body_partial_json(json!({
            "payload": {
                "capability": {
                    "type": "devices.capabilities.range",
                    "instance": "brightness",
                    "value": 100
                }
            }
        })))
        .respond_with(success_control_response())
        .expect(1)
        .mount(&server)
        .await;

    let client = client_for(&server).await;
    client.set_brightness(DEVICE, SKU, 250).await.unwrap();
}

#[tokio::test]
async fn set_color_sends_packed_rgb() {
    let server = MockServer::start().await;

    // (255 << 16) | (0 << 8) | 255 == 16711935
    Mock::given(method("POST"))
        .and(path("/router/api/v1/device/control"))
        .and(body_partial_json(json!({
            "payload": {
                "capability": {
                    "type": "devices.capabilities.color_setting",
                    "instance": "colorRgb",
                    "value": 16711935
                }
            }
        })))
        .respond_with(success_control_response())
        .expect(1)
        .mount(&server)
        .await;

    let client = client_for(&server).await;
    client
        .set_color(DEVICE, SKU, Color::new(255, 0, 255))
        .await
        .unwrap();
}

#[tokio::test]
async fn set_color_temperature_clamps_to_range() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/router/api/v1/device/control"))
        .and(body_partial_json(json!({
            "payload": {
                "capability": {
                    "type": "devices.capabilities.color_setting",
                    "instance": "colorTemperatureK",
                    "value": 9000
                }
            }
        })))
        .respond_with(success_control_response())
        .expect(1)
        .mount(&server)
        .await;

    let client = client_for(&server).await;
    client
        .set_color_temperature(DEVICE, SKU, 20000)
        .await
        .unwrap();
}

// --- scenes ------------------------------------------------------------

#[tokio::test]
async fn get_dynamic_scenes_happy_path() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/router/api/v1/device/scenes"))
        .and(body_partial_json(
            json!({ "payload": { "sku": SKU, "device": DEVICE } }),
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "requestId": "uuid",
            "msg": "success",
            "code": 200,
            "payload": {
                "sku": SKU,
                "device": DEVICE,
                "capabilities": [
                    {
                        "type": "devices.capabilities.dynamic_scene",
                        "instance": "lightScene",
                        "parameters": {
                            "dataType": "ENUM",
                            "options": [
                                { "name": "Sunrise", "value": { "paramId": 4280, "id": 3853 } },
                                { "name": "Sunset", "value": { "paramId": 4281, "id": 3854 } }
                            ]
                        }
                    }
                ]
            }
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = client_for(&server).await;
    let scenes = client.get_dynamic_scenes(DEVICE, SKU).await.unwrap();

    assert_eq!(scenes.len(), 2);
    assert_eq!(scenes[0].name, "Sunrise");
    assert_eq!(scenes[0].id, 3853);
    assert_eq!(scenes[0].param_id, Some(4280));
    assert_eq!(
        scenes[0].capability_type,
        "devices.capabilities.dynamic_scene"
    );
    assert_eq!(scenes[0].instance, "lightScene");
}

#[tokio::test]
async fn get_diy_scenes_happy_path() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/router/api/v1/device/diy-scenes"))
        .and(body_partial_json(
            json!({ "payload": { "sku": SKU, "device": DEVICE } }),
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "requestId": "uuid",
            "msg": "success",
            "code": 200,
            "payload": {
                "sku": SKU,
                "device": DEVICE,
                "capabilities": [
                    {
                        "type": "devices.capabilities.diy_color_setting",
                        "instance": "diyScene",
                        "parameters": {
                            "dataType": "ENUM",
                            "options": [
                                { "name": "Xmas lights 2", "value": 8216931 },
                                { "name": "test", "value": 8216643 }
                            ]
                        }
                    }
                ]
            }
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = client_for(&server).await;
    let scenes = client.get_diy_scenes(DEVICE, SKU).await.unwrap();

    assert_eq!(scenes.len(), 2);
    assert_eq!(scenes[0].name, "Xmas lights 2");
    assert_eq!(scenes[0].id, 8216931);
    assert_eq!(scenes[0].param_id, None);
    assert_eq!(
        scenes[0].capability_type,
        "devices.capabilities.diy_color_setting"
    );
    assert_eq!(scenes[0].instance, "diyScene");
}

#[tokio::test]
async fn get_diy_scenes_empty_capabilities() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/router/api/v1/device/diy-scenes"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "requestId": "uuid",
            "msg": "success",
            "code": 200,
            "payload": { "sku": SKU, "device": DEVICE, "capabilities": [] }
        })))
        .mount(&server)
        .await;

    let client = client_for(&server).await;
    let scenes = client.get_diy_scenes(DEVICE, SKU).await.unwrap();
    assert!(scenes.is_empty());
}

#[tokio::test]
async fn set_scene_dynamic_sends_param_id_and_id() {
    let server = MockServer::start().await;

    // Per the docs: value is {"paramId": 4280, "id": 3853}
    Mock::given(method("POST"))
        .and(path("/router/api/v1/device/control"))
        .and(body_partial_json(json!({
            "payload": {
                "sku": SKU,
                "device": DEVICE,
                "capability": {
                    "type": "devices.capabilities.dynamic_scene",
                    "instance": "lightScene",
                    "value": { "paramId": 4280, "id": 3853 }
                }
            }
        })))
        .respond_with(success_control_response())
        .expect(1)
        .mount(&server)
        .await;

    let scene = Scene {
        name: "Sunrise".to_string(),
        id: 3853,
        param_id: Some(4280),
        capability_type: "devices.capabilities.dynamic_scene".to_string(),
        instance: "lightScene".to_string(),
    };

    let client = client_for(&server).await;
    client.set_scene(DEVICE, SKU, &scene).await.unwrap();
}

#[tokio::test]
async fn set_scene_diy_sends_bare_integer_value() {
    let server = MockServer::start().await;

    // DIY scene option values are bare integers, echoed back with the
    // diy_color_setting capability type from the diy-scenes response.
    Mock::given(method("POST"))
        .and(path("/router/api/v1/device/control"))
        .and(body_partial_json(json!({
            "payload": {
                "capability": {
                    "type": "devices.capabilities.diy_color_setting",
                    "instance": "diyScene",
                    "value": 8216931
                }
            }
        })))
        .respond_with(success_control_response())
        .expect(1)
        .mount(&server)
        .await;

    let scene = Scene {
        name: "Xmas lights 2".to_string(),
        id: 8216931,
        param_id: None,
        capability_type: "devices.capabilities.diy_color_setting".to_string(),
        instance: "diyScene".to_string(),
    };

    let client = client_for(&server).await;
    client.set_scene(DEVICE, SKU, &scene).await.unwrap();
}

// --- segments ----------------------------------------------------------

#[tokio::test]
async fn set_segment_color_sends_segment_array_and_packed_rgb() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/router/api/v1/device/control"))
        .and(body_partial_json(json!({
            "payload": {
                "capability": {
                    "type": "devices.capabilities.segment_color_setting",
                    "instance": "segmentedColorRgb",
                    "value": { "segment": [0, 1, 2, 3], "rgb": 255 }
                }
            }
        })))
        .respond_with(success_control_response())
        .expect(1)
        .mount(&server)
        .await;

    let client = client_for(&server).await;
    client
        .set_segment_color(DEVICE, SKU, &[0, 1, 2, 3], 0, 0, 255)
        .await
        .unwrap();
}

#[tokio::test]
async fn set_segment_brightness_sends_segment_array_and_brightness() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/router/api/v1/device/control"))
        .and(body_partial_json(json!({
            "payload": {
                "capability": {
                    "type": "devices.capabilities.segment_color_setting",
                    "instance": "segmentedBrightness",
                    "value": { "segment": [2, 5], "brightness": 50 }
                }
            }
        })))
        .respond_with(success_control_response())
        .expect(1)
        .mount(&server)
        .await;

    let client = client_for(&server).await;
    client
        .set_segment_brightness(DEVICE, SKU, &[2, 5], 50)
        .await
        .unwrap();
}

// --- error handling ------------------------------------------------------

#[tokio::test]
async fn unauthorized_maps_to_invalid_api_key() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/router/api/v1/user/devices"))
        .respond_with(ResponseTemplate::new(401))
        .expect(1) // no retry on 401
        .mount(&server)
        .await;

    let client = client_for(&server).await;
    let err = client.get_devices().await.unwrap_err();

    assert!(matches!(err, Error::InvalidApiKey));
    assert!(err.to_string().contains("invalid API key"));
}

#[tokio::test]
async fn rate_limit_with_retry_after_is_not_retried() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/router/api/v1/user/devices"))
        .respond_with(ResponseTemplate::new(429).insert_header("Retry-After", "30"))
        .expect(1) // must NOT be blind-retried
        .mount(&server)
        .await;

    let client = client_for(&server).await;
    let err = client.get_devices().await.unwrap_err();

    // Human-readable message, no raw body dump
    assert!(err.to_string().contains("rate limited"));
    assert!(err.to_string().contains("retry after 30s"));
    match err {
        Error::RateLimited { retry_after_secs } => assert_eq!(retry_after_secs, Some(30)),
        other => panic!("expected RateLimited, got {other:?}"),
    }
}

#[tokio::test]
async fn rate_limit_without_headers_has_no_retry_after() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/router/api/v1/device/control"))
        .respond_with(ResponseTemplate::new(429))
        .expect(1)
        .mount(&server)
        .await;

    let client = client_for(&server).await;
    let err = client.turn_on(DEVICE, SKU).await.unwrap_err();

    match err {
        Error::RateLimited { retry_after_secs } => {
            assert_eq!(retry_after_secs, None);
        }
        other => panic!("expected RateLimited, got {other:?}"),
    }
}

#[tokio::test]
async fn server_error_is_retried_until_success() {
    let server = MockServer::start().await;

    // First two attempts fail with 500, then the request succeeds.
    Mock::given(method("GET"))
        .and(path("/router/api/v1/user/devices"))
        .respond_with(ResponseTemplate::new(500))
        .up_to_n_times(2)
        .expect(2)
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/router/api/v1/user/devices"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "code": 200,
            "message": "success",
            "data": []
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = client_for(&server).await;
    let devices = client.get_devices().await.unwrap();
    assert!(devices.is_empty());
}

#[tokio::test]
async fn server_error_exhausts_retries() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/router/api/v1/user/devices"))
        .respond_with(ResponseTemplate::new(503))
        .expect(4) // initial attempt + 3 retries
        .mount(&server)
        .await;

    let client = client_for(&server).await;
    let err = client.get_devices().await.unwrap_err();

    match err {
        Error::Server { status } => assert_eq!(status, 503),
        other => panic!("expected Server error, got {other:?}"),
    }
}

#[tokio::test]
async fn no_retries_when_configured_with_zero_attempts() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/router/api/v1/user/devices"))
        .respond_with(ResponseTemplate::new(500))
        .expect(1)
        .mount(&server)
        .await;

    let client = client_without_retries(&server).await;
    let err = client.get_devices().await.unwrap_err();
    assert!(matches!(err, Error::Server { status: 500 }));
}

#[tokio::test]
async fn timeout_is_reported_as_transport_error() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/router/api/v1/user/devices"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(json!({ "code": 200, "message": "success", "data": [] }))
                .set_delay(Duration::from_secs(2)),
        )
        .mount(&server)
        .await;

    let client = GoveeClient::with_config(
        "test-api-key",
        ClientConfig {
            timeout: Duration::from_millis(100),
            retry_attempts: 0,
            base_url: server.uri(),
        },
    );

    let err = client.get_devices().await.unwrap_err();
    match err {
        Error::Request(e) => assert!(e.is_timeout(), "expected timeout, got {e:?}"),
        other => panic!("expected Request(timeout), got {other:?}"),
    }
}

#[tokio::test]
async fn api_level_error_code_maps_to_api_error() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/router/api/v1/device/control"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "requestId": "uuid",
            "msg": "device offline",
            "code": 400
        })))
        .mount(&server)
        .await;

    let client = client_for(&server).await;
    let err = client.turn_on(DEVICE, SKU).await.unwrap_err();

    match err {
        Error::Api { code, message } => {
            assert_eq!(code, 400);
            assert_eq!(message, "device offline");
        }
        other => panic!("expected Api error, got {other:?}"),
    }
}
