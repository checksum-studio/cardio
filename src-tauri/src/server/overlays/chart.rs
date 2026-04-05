use axum::response::Html;

#[utoipa::path(
    get,
    path = "/overlay/chart",
    tag = "Overlays",
    summary = "Rolling HR chart overlay",
    description = "OBS browser source. 60-second rolling HR graph with spline interpolation, color-coded by zone.\n\
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
pub async fn overlay_chart_page() -> Html<String> {
    Html(OVERLAY_CHART_HTML.to_string())
}

const OVERLAY_CHART_HTML: &str = r##"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>Cardio Chart</title>
<style>
  *, *::before, *::after { margin: 0; padding: 0; box-sizing: border-box; }
  html, body {
    background: transparent !important;
    width: 100%;
    height: 100%;
    overflow: hidden;
  }
  canvas {
    display: block;
    width: 100%;
    height: 100%;
  }
</style>
</head>
<body>
<canvas id="c"></canvas>
<script>
(function() {
  var canvas = document.getElementById('c');
  var ctx = canvas.getContext('2d');

  var ws = null;
  var reconnectTimer = null;
  var staleTimer = null;
  var connected = false;

  var WINDOW_SEC = 60;
  var data = [];

  function parseColor(hex) {
    if (!hex) return null;
    hex = hex.replace(/^#/, '');
    if (hex.length === 3) hex = hex.split('').map(function(c){return c+c;}).join('');
    if (hex.length !== 6) return null;
    var num = parseInt(hex, 16);
    if (isNaN(num)) return null;
    return [ (num >> 16) & 255, (num >> 8) & 255, num & 255 ];
  }
  
  var params = new URLSearchParams(window.location.search);
  var customColor = parseColor(params.get('color'));

  var zones = [
    { bpm: 0,   r: 220, g: 180, b: 180 },
    { bpm: 80,  r: 220, g: 180, b: 180 },
    { bpm: 100, r: 120, g: 220, b: 160 },
    { bpm: 120, r: 250, g: 204, b: 21  },
    { bpm: 145, r: 249, g: 115, b: 22  },
    { bpm: 170, r: 239, g: 68,  b: 68  },
    { bpm: 200, r: 239, g: 68,  b: 68  },
  ];

  function lerp(a, b, t) { return a + (b - a) * t; }
  function clamp01(t) { return t < 0 ? 0 : t > 1 ? 1 : t; }

  function zoneColor(bpm) {
    for (var i = 0; i < zones.length - 1; i++) {
      if (bpm <= zones[i + 1].bpm) {
        var lo = zones[i], hi = zones[i + 1];
        var t = hi.bpm === lo.bpm ? 0 : clamp01((bpm - lo.bpm) / (hi.bpm - lo.bpm));
        return [
          Math.round(lerp(lo.r, hi.r, t)),
          Math.round(lerp(lo.g, hi.g, t)),
          Math.round(lerp(lo.b, hi.b, t))
        ];
      }
    }
    var z = zones[zones.length - 1];
    return [z.r, z.g, z.b];
  }

  function rgba(c, a) { return 'rgba(' + c[0] + ',' + c[1] + ',' + c[2] + ',' + a + ')'; }

  var dpr = window.devicePixelRatio || 1;
  function resize() {
    var r = canvas.getBoundingClientRect();
    canvas.width = r.width * dpr;
    canvas.height = r.height * dpr;
    ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
  }
  window.addEventListener('resize', resize);
  resize();

  function catmullRomToBezier(p0, p1, p2, p3) {
    var alpha = 0.5;
    function dist(a, b) {
      var dx = b[0] - a[0], dy = b[1] - a[1];
      return Math.sqrt(dx * dx + dy * dy);
    }
    var d01 = Math.pow(dist(p0, p1), alpha);
    var d12 = Math.pow(dist(p1, p2), alpha);
    var d23 = Math.pow(dist(p2, p3), alpha);

    if (d01 < 1e-6) d01 = 1;
    if (d12 < 1e-6) d12 = 1;
    if (d23 < 1e-6) d23 = 1;

    var b1x = (d01 * d01 * p2[0] - d12 * d12 * p0[0] + (2 * d01 * d01 + 3 * d01 * d12 + d12 * d12) * p1[0]) / (3 * d01 * (d01 + d12));
    var b1y = (d01 * d01 * p2[1] - d12 * d12 * p0[1] + (2 * d01 * d01 + 3 * d01 * d12 + d12 * d12) * p1[1]) / (3 * d01 * (d01 + d12));

    var b2x = (d23 * d23 * p1[0] - d12 * d12 * p3[0] + (2 * d23 * d23 + 3 * d23 * d12 + d12 * d12) * p2[0]) / (3 * d23 * (d23 + d12));
    var b2y = (d23 * d23 * p1[1] - d12 * d12 * p3[1] + (2 * d23 * d23 + 3 * d23 * d12 + d12 * d12) * p2[1]) / (3 * d23 * (d23 + d12));

    return [[b1x, b1y], [b2x, b2y]];
  }

  function traceCatmullRom(pts, continued) {
    if (pts.length < 2) return;
    if (pts.length === 2) {
      if (continued) { ctx.lineTo(pts[0][0], pts[0][1]); }
      else { ctx.moveTo(pts[0][0], pts[0][1]); }
      ctx.lineTo(pts[1][0], pts[1][1]);
      return;
    }

    if (continued) { ctx.lineTo(pts[0][0], pts[0][1]); }
    else { ctx.moveTo(pts[0][0], pts[0][1]); }

    for (var i = 0; i < pts.length - 1; i++) {
      var p0 = pts[i === 0 ? 0 : i - 1];
      var p1 = pts[i];
      var p2 = pts[i + 1];
      var p3 = pts[i + 2 < pts.length ? i + 2 : pts.length - 1];

      var cp = catmullRomToBezier(p0, p1, p2, p3);
      ctx.bezierCurveTo(cp[0][0], cp[0][1], cp[1][0], cp[1][1], p2[0], p2[1]);
    }
  }

  function draw() {
    var w = canvas.width / dpr;
    var h = canvas.height / dpr;
    ctx.clearRect(0, 0, w, h);

    var mx = w * 0.02;
    var mt = h * 0.08;
    var mb = h * 0.06;
    var plotL = mx;
    var plotR = w - mx;
    var plotT = mt;
    var plotB = h - mb;
    var plotW = plotR - plotL;
    var plotH = plotB - plotT;

    var now = Date.now();
    var cutoff = now - WINDOW_SEC * 1000;
    while (data.length > 0 && data[0].t < cutoff) data.shift();

    if (data.length < 2) {
      ctx.strokeStyle = 'rgba(255,255,255,0.04)';
      ctx.lineWidth = 1;
      ctx.beginPath();
      ctx.moveTo(plotL, plotB);
      ctx.lineTo(plotR, plotB);
      ctx.stroke();
      requestAnimationFrame(draw);
      return;
    }

    var yMin = 300, yMax = 0;
    for (var i = 0; i < data.length; i++) {
      if (data[i].bpm < yMin) yMin = data[i].bpm;
      if (data[i].bpm > yMax) yMax = data[i].bpm;
    }
    var pad = Math.max(8, (yMax - yMin) * 0.2);
    yMin = Math.floor(yMin - pad);
    yMax = Math.ceil(yMax + pad);
    if (yMin < 30) yMin = 30;
    if (yMax - yMin < 15) yMax = yMin + 15;
    var yRange = yMax - yMin;

    function tx(t) { return plotL + ((t - cutoff) / (WINDOW_SEC * 1000)) * plotW; }
    function ty(bpm) { return plotB - ((bpm - yMin) / yRange) * plotH; }

    var pts = [];
    for (var i = 0; i < data.length; i++) {
      pts.push([tx(data[i].t), ty(data[i].bpm)]);
    }
    
    // Draw background grid
    ctx.strokeStyle = 'rgba(255,255,255,0.05)';
    ctx.lineWidth = 1;
    ctx.beginPath();
    var gridStep = 20;
    var firstGrid = Math.ceil(yMin / gridStep) * gridStep;
    for (var gy = firstGrid; gy <= yMax; gy += gridStep) {
      var yPos = ty(gy);
      ctx.moveTo(plotL, yPos);
      ctx.lineTo(plotR, yPos);
    }
    ctx.stroke();

    var lastColor = customColor || zoneColor(data[data.length - 1].bpm);
    var grad = ctx.createLinearGradient(0, plotT, 0, plotB);
    grad.addColorStop(0, rgba(lastColor, 0.40));
    grad.addColorStop(0.7, rgba(lastColor, 0.05));
    grad.addColorStop(1, rgba(lastColor, 0.0));

    ctx.beginPath();
    ctx.moveTo(pts[0][0], plotB);
    traceCatmullRom(pts, true);
    ctx.lineTo(pts[pts.length - 1][0], plotB);
    ctx.closePath();
    ctx.fillStyle = grad;
    ctx.fill();

    var lineW = Math.max(2, Math.min(h * 0.018, 4));
    ctx.lineWidth = lineW;
    ctx.lineJoin = 'round';
    ctx.lineCap = 'round';
    ctx.strokeStyle = rgba(lastColor, 0.9);

    ctx.beginPath();
    traceCatmullRom(pts, false);
    ctx.stroke();

    ctx.lineWidth = lineW * 3;
    ctx.strokeStyle = rgba(lastColor, 0.15);
    ctx.beginPath();
    traceCatmullRom(pts, false);
    ctx.stroke();

    var last = pts[pts.length - 1];
    var dotR = Math.max(3, h * 0.025);
    
    // Pulsing echo dot
    var beatProgress = (now % 1000) / 1000;
    ctx.beginPath();
    ctx.arc(last[0], last[1], dotR * (1 + beatProgress * 3), 0, Math.PI * 2);
    ctx.fillStyle = rgba(lastColor, 0.3 * (1 - beatProgress));
    ctx.fill();

    ctx.beginPath();
    ctx.arc(last[0], last[1], dotR * 1.5, 0, Math.PI * 2);
    ctx.fillStyle = rgba(lastColor, 0.2);
    ctx.fill();

    ctx.beginPath();
    ctx.arc(last[0], last[1], dotR, 0, Math.PI * 2);
    ctx.fillStyle = rgba(lastColor, 1);
    ctx.fill();

    var fontSize = Math.round(h * 0.22);
    ctx.font = '800 ' + fontSize + 'px "Inter", "Segoe UI", system-ui, sans-serif';
    ctx.textAlign = 'right';
    ctx.textBaseline = 'top';
    ctx.fillStyle = '#ffffff';
    ctx.shadowColor = rgba(lastColor, 0.8);
    ctx.shadowBlur = h * 0.04;
    ctx.fillText(data[data.length - 1].bpm, w - mx, h * 0.03);
    
    var lblSize = Math.round(h * 0.08);
    ctx.font = '600 ' + lblSize + 'px "Inter", "Segoe UI", system-ui, sans-serif';
    ctx.fillStyle = rgba(lastColor, 0.8);
    ctx.shadowBlur = 0;
    ctx.fillText("BPM", w - mx - ctx.measureText(data[data.length-1].bpm).width - mx, h * 0.03 + (fontSize - lblSize));

    var fadeW = plotW * 0.08;
    var fadeGrad = ctx.createLinearGradient(plotL, 0, plotL + fadeW, 0);
    fadeGrad.addColorStop(0, 'rgba(0,0,0,1)');
    fadeGrad.addColorStop(1, 'rgba(0,0,0,0)');
    ctx.globalCompositeOperation = 'destination-out';
    ctx.fillStyle = fadeGrad;
    ctx.fillRect(0, 0, plotL + fadeW, h);
    ctx.globalCompositeOperation = 'source-over';

    requestAnimationFrame(draw);
  }

  requestAnimationFrame(draw);

  function getWsUrl() {
    var params = new URLSearchParams(window.location.search);
    var host = params.get('host') || window.location.hostname;
    var port = params.get('port') || window.location.port;
    if (host === 'localhost') host = '127.0.0.1';
    var proto = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    return proto + '//' + host + (port ? ':' + port : '') + '/api/ws';
  }

  function onData(bpm) {
    data.push({ t: Date.now(), bpm: bpm });
    connected = true;
    clearTimeout(staleTimer);
    staleTimer = setTimeout(function() { connected = false; }, 10000);
  }

  function connect() {
    if (ws && (ws.readyState === WebSocket.OPEN || ws.readyState === WebSocket.CONNECTING)) return;
    try { ws = new WebSocket(getWsUrl()); } catch(e) { scheduleReconnect(); return; }
    ws.onopen = function() { clearTimeout(reconnectTimer); };
    ws.onmessage = function(e) {
      try {
        var d = JSON.parse(e.data);
        if (d.heart_rate && d.heart_rate > 0) onData(d.heart_rate);
      } catch(x) {}
    };
    ws.onclose = function() { connected = false; scheduleReconnect(); };
    ws.onerror = function() { ws.close(); };
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
