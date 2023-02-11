#![cfg(windows)]

use windows::core::PCWSTR;
use windows::w;

mod sys;

const SERVICE_NAME: PCWSTR = w!("cf-ddns");
const SERVICE_DESCRIPTION: PCWSTR = w!("Cloudflare DDNS");

fn install() {}
