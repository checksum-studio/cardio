use axum::response::Html;

#[utoipa::path(
    get,
    path = "/overlay",
    tag = "Overlays",
    summary = "Heart + BPM overlay",
    description = "OBS browser source. Beating heart icon and BPM, color-coded by HR zone, synced to heartbeat.\n\
        Add as a Browser Source with transparent background. Streams via `/api/ws`.",
    params(
        ("host" = Option<String>, Query, description = "WebSocket hostname override"),
        ("port" = Option<u16>, Query, description = "WebSocket port override"),
        ("color" = Option<String>, Query, description = "Optional hex color override (e.g., 'ff0000' or '#ff0000')")
    ),
    responses(
        (status = 200, description = "HTML page", content_type = "text/html")
    )
)]
pub async fn overlay_page() -> Html<String> {
    Html(OVERLAY_HTML.to_string())
}

const OVERLAY_HTML: &str = r##"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>Cardio Overlay</title>
<style>
  *, *::before, *::after { margin: 0; padding: 0; box-sizing: border-box; }

  html, body {
    background: transparent !important;
    width: 100%;
    height: 100%;
    overflow: hidden;
    font-family: 'Inter', 'Segoe UI', system-ui, -apple-system, sans-serif;
    -webkit-font-smoothing: antialiased;
    -moz-osx-font-smoothing: grayscale;
  }

  body {
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .cardio-widget {
    display: inline-flex;
    align-items: center;
    gap: 4vh;
    transition: opacity 0.5s ease, filter 0.5s ease;
  }

  .cardio-widget.disconnected {
    opacity: 0.4;
    filter: grayscale(100%);
  }

  .heart-wrap {
    position: relative;
    height: 60vh;
    width: 60vh;
    flex-shrink: 0;
    display: flex;
    align-items: center;
    justify-content: center;
  }
  
  .heart-echo {
    position: absolute;
    top: 0; left: 0; right: 0; bottom: 0;
    border-radius: 50%;
    background: currentColor;
    opacity: 0;
    z-index: -1;
    transform: scale(0.8);
    filter: blur(2vh);
    transition: color 1s ease;
  }

  .heart-icon {
    height: 100%;
    width: 100%;
    display: block;
    color: #dcb4b4;
    filter: drop-shadow(0 1vh 3vh var(--heart-glow, rgba(220, 180, 180, 0.4)));
    transform-origin: center center;
    will-change: transform, filter;
    transition: color 1s ease, filter 1s ease;
    z-index: 1;
  }

  .beating .heart-icon {
    animation: heartbeat var(--beat-dur, 857ms) cubic-bezier(0.4, 0, 0.2, 1) infinite;
  }
  
  .beating .heart-echo {
    animation: echo var(--beat-dur, 857ms) cubic-bezier(0.4, 0, 0.2, 1) infinite;
  }

  @keyframes heartbeat {
    0%   { transform: scale(1); }
    15%  { transform: scale(1.18); }
    30%  { transform: scale(1); }
    45%  { transform: scale(1.08); }
    60%  { transform: scale(1); }
    100% { transform: scale(1); }
  }
  
  @keyframes echo {
    0%   { transform: scale(0.8); opacity: 0.5; }
    50%  { transform: scale(1.5); opacity: 0; }
    100% { transform: scale(1.5); opacity: 0; }
  }

  .bpm-value {
    font-size: 68vh;
    font-weight: 800;
    line-height: 1;
    letter-spacing: -0.03em;
    font-variant-numeric: tabular-nums;
    color: #ffffff;
    transition: color 1s ease, text-shadow 1s ease;
    text-shadow: 0 1vh 4vh rgba(0, 0, 0, 0.8), 0 0.5vh 1vh rgba(0, 0, 0, 0.5);
  }
</style>
</head>
<body>

<div class="cardio-widget disconnected" id="widget">
  <div class="heart-wrap" id="heart-wrap">
    <div class="heart-echo" id="echo"></div>
    <svg class="heart-icon" id="heart" viewBox="0 0 24 24" fill="currentColor" xmlns="http://www.w3.org/2000/svg">
      <path d="M11.645 20.91l-.007-.003-.022-.012a15.247 15.247 0 01-.383-.218 25.18 25.18 0 01-4.244-3.17C4.688 15.36 2.25 12.174 2.25 8.25 2.25 5.322 4.714 3 7.688 3A5.5 5.5 0 0112 5.052 5.5 5.5 0 0116.313 3c2.973 0 5.437 2.322 5.437 5.25 0 3.925-2.438 7.111-4.739 9.256a25.175 25.175 0 01-4.244 3.17 15.247 15.247 0 01-.383.219l-.022.012-.007.004-.003.001a.752.752 0 01-.704 0l-.003-.001z"/>
    </svg>
  </div>
  <span class="bpm-value" id="bpm">--</span>
</div>

<script>
(function() {
  var widget = document.getElementById('widget');
  var bpmEl  = document.getElementById('bpm');
  var heartEl = document.getElementById('heart');
  var heartWrap = document.getElementById('heart-wrap');
  var echoEl = document.getElementById('echo');

  var ws = null;
  var reconnectTimer = null;
  var staleTimer = null;

  function parseColor(hex) {
    if (!hex) return null;
    hex = hex.replace(/^#/, '');
    if (hex.length === 3) hex = hex.split('').map(function(c){return c+c;}).join('');
    if (hex.length !== 6) return null;
    var num = parseInt(hex, 16);
    if (isNaN(num)) return null;
    return { r: (num >> 16) & 255, g: (num >> 8) & 255, b: num & 255, glow: 0.6 };
  }
  
  var params = new URLSearchParams(window.location.search);
  var customColor = parseColor(params.get('color'));

  var zones = [
    { bpm: 0,   color: [220, 180, 180], glow: 0.25 },
    { bpm: 80,  color: [220, 180, 180], glow: 0.25 },
    { bpm: 100, color: [120, 220, 160], glow: 0.35 },
    { bpm: 120, color: [250, 204, 21],  glow: 0.4  },
    { bpm: 145, color: [249, 115, 22],  glow: 0.55 },
    { bpm: 170, color: [239, 68, 68],   glow: 0.7  },
    { bpm: 200, color: [239, 68, 68],   glow: 0.9  },
  ];

  function lerp(a, b, t) {
    return a + (b - a) * t;
  }

  function getZoneColor(bpm) {
    for (var i = 0; i < zones.length - 1; i++) {
      if (bpm <= zones[i + 1].bpm) {
        var lo = zones[i];
        var hi = zones[i + 1];
        var range = hi.bpm - lo.bpm;
        var t = range > 0 ? (bpm - lo.bpm) / range : 0;
        t = Math.max(0, Math.min(1, t));
        var r = Math.round(lerp(lo.color[0], hi.color[0], t));
        var g = Math.round(lerp(lo.color[1], hi.color[1], t));
        var b = Math.round(lerp(lo.color[2], hi.color[2], t));
        var glow = lerp(lo.glow, hi.glow, t);
        return { r: r, g: g, b: b, glow: glow };
      }
    }
    var last = zones[zones.length - 1];
    return { r: last.color[0], g: last.color[1], b: last.color[2], glow: last.glow };
  }

  function getWsUrl() {
    var params = new URLSearchParams(window.location.search);
    var host = params.get('host') || window.location.hostname;
    var port = params.get('port') || window.location.port;
    if (host === 'localhost') host = '127.0.0.1';
    var proto = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    return proto + '//' + host + (port ? ':' + port : '') + '/api/ws';
  }

  function updateBpm(bpm) {
    bpmEl.textContent = bpm;
    widget.classList.remove('disconnected');
    heartWrap.classList.add('beating');

    var dur = Math.round((60 / bpm) * 1000);
    heartWrap.style.setProperty('--beat-dur', dur + 'ms');

    var zone = customColor || getZoneColor(bpm);
    var rgb = 'rgb(' + zone.r + ',' + zone.g + ',' + zone.b + ')';
    var glowRgba = 'rgba(' + zone.r + ',' + zone.g + ',' + zone.b + ',' + zone.glow.toFixed(2) + ')';

    heartEl.style.color = rgb;
    echoEl.style.color = rgb;
    heartEl.style.setProperty('--heart-glow', glowRgba);
    bpmEl.style.color = rgb;

    clearTimeout(staleTimer);
    staleTimer = setTimeout(function() {
      setDisconnected();
    }, 10000);
  }

  function setDisconnected() {
    widget.classList.add('disconnected');
    heartWrap.classList.remove('beating');
    bpmEl.textContent = '--';
    clearTimeout(staleTimer);
  }

  function connect() {
    if (ws && (ws.readyState === WebSocket.OPEN || ws.readyState === WebSocket.CONNECTING)) {
      return;
    }

    try {
      ws = new WebSocket(getWsUrl());
    } catch (e) {
      scheduleReconnect();
      return;
    }

    ws.onopen = function() {
      clearTimeout(reconnectTimer);
    };

    ws.onmessage = function(event) {
      try {
        var data = JSON.parse(event.data);
        if (data.heart_rate && data.heart_rate > 0) {
          updateBpm(data.heart_rate);
        }
      } catch (e) {}
    };

    ws.onclose = function() {
      setDisconnected();
      scheduleReconnect();
    };

    ws.onerror = function() {
      ws.close();
    };
  }

  function scheduleReconnect() {
    clearTimeout(reconnectTimer);
    reconnectTimer = setTimeout(connect, 3000);
  }

  connect();
})();
</script>
</body>
</html>
"##;
