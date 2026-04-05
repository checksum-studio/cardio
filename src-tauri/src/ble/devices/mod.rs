use super::device::DeviceType;

pub fn identify_device(name: &str) -> DeviceType {
    let name_lower = name.to_lowercase();

    if name_lower.contains("polar h10") {
        DeviceType::PolarH10
    } else if name_lower.contains("polar h9") {
        DeviceType::PolarH9
    } else if name_lower.contains("polar oh1") {
        DeviceType::PolarOh1
    } else if name_lower.contains("verity sense") || name_lower.contains("polar sense") {
        DeviceType::PolarVeritySense
    } else if name_lower.contains("polar") {
        DeviceType::PolarGeneric
    } else if name_lower.contains("hrm-pro") {
        DeviceType::GarminHrmPro
    } else if name_lower.contains("hrm-dual") {
        DeviceType::GarminHrmDual
    } else if name_lower.contains("hrm-fit") {
        DeviceType::GarminHrmFit
    } else if name_lower.starts_with("hrm-") {
        DeviceType::GarminGeneric
    } else if name_lower.contains("tickr x") {
        DeviceType::WahooTickrX
    } else if name_lower.contains("tickr fit") {
        DeviceType::WahooTickrFit
    } else if name_lower.contains("tickr") {
        DeviceType::WahooTickr
    } else if name_lower.contains("trackr") {
        DeviceType::WahooGeneric
    } else if name_lower.contains("suunto smart sensor") {
        DeviceType::SuuntoSmartSensor
    } else if name_lower.contains("suunto") {
        DeviceType::SuuntoGeneric
    } else if name_lower.contains("h808s") {
        DeviceType::CoospoH808s
    } else if name_lower.contains("hw807") {
        DeviceType::CoospoHw807
    } else if (name_lower.contains("coospo") && name_lower.contains("h6"))
        || name_lower.starts_with("h6-")
    {
        DeviceType::CoospoH6
    } else if name_lower.contains("coospo") {
        DeviceType::CoospoGeneric
    } else if name_lower.contains("h303") {
        DeviceType::MageneH303
    } else if name_lower.contains("h64") {
        DeviceType::MageneH64
    } else if name_lower.contains("magene") {
        DeviceType::MageneGeneric
    } else if name_lower.contains("whoop") {
        DeviceType::Whoop
    } else if name_lower.contains("movesense") {
        DeviceType::Movesense
    } else if name_lower.contains("rhythm") || name_lower.contains("scosche") {
        DeviceType::ScoscheRhythm
    } else if name_lower.starts_with("mz-") || name_lower.contains("myzone") {
        DeviceType::Myzone
    } else if name_lower.contains("viiiiva") {
        DeviceType::Viiiiva
    } else if name_lower.contains("moofit") || name_lower.contains("hr8") {
        DeviceType::Moofit
    } else {
        DeviceType::Generic
    }
}
