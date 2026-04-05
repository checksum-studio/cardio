use axum::response::Html;

#[utoipa::path(
    get,
    path = "/overlay/ring",
    tag = "Overlays",
    summary = "Circular progress ring overlay",
    description = "OBS browser source. Minimalist circular ring that fills based on HR.

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
pub async fn overlay_ring_page() -> Html<String> {
    Html(OVERLAY_RING_HTML.to_string())
}

const OVERLAY_RING_HTML: &str = r##"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>Cardio Ring</title>
<style>
  *, *::before, *::after { margin: 0; padding: 0; box-sizing: border-box; }
  html, body {
    background: transparent !important;
    width: 100%; height: 100%; overflow: hidden;
    font-family: 'Segoe UI', system-ui, sans-serif;
  }
  body { display: flex; align-items: center; justify-content: center; }
  .ring-container {
    position: relative; width: 90vh; height: 90vh;
    display: flex; align-items: center; justify-content: center;
    transition: opacity 0.5s, filter 0.5s;
  }
  .ring-container.disconnected { opacity: 0.3; filter: grayscale(100%); }
  svg.ring {
    position: absolute; top: 0; left: 0; width: 100%; height: 100%;
    transform: rotate(-90deg); overflow: visible;
  }
  .bg-ring { fill: none; stroke: rgba(255,255,255,0.1); stroke-width: 6; }
  .fg-ring {
    fill: none; stroke: currentColor; stroke-width: 6; stroke-linecap: round;
    transition: stroke-dashoffset 0.5s ease-out, stroke 1s ease, filter 1s ease;
    filter: drop-shadow(0 0 1.5vh var(--glow-color, rgba(220,180,180,0.5)));
  }
  .content {
    position: relative; z-index: 10; display: flex; flex-direction: column; align-items: center;
    color: white; text-shadow: 0 0.5vh 2vh rgba(0,0,0,0.8);
  }
  .bpm-val { font-size: 28vh; font-weight: 800; line-height: 1; font-variant-numeric: tabular-nums; }
  .bpm-label { font-size: 7vh; font-weight: 700; opacity: 0.8; letter-spacing: 0.1em; margin-top: -1vh; }
  .heart-icon { width: 10vh; height: 10vh; margin-bottom: 2vh; color: currentColor; transition: color 1s ease; filter: drop-shadow(0 0 1vh var(--glow-color, rgba(220,180,180,0.5))); }
  .beating .heart-icon { animation: throb var(--beat-dur, 857ms) cubic-bezier(0.4, 0, 0.2, 1) infinite; }
  @keyframes throb {
    0% { transform: scale(1); } 15% { transform: scale(1.18); }
    30% { transform: scale(1); } 45% { transform: scale(1.08); } 60% { transform: scale(1); }
  }
</style>
</head>
<body>
<div class="ring-container disconnected" id="widget">
  <svg class="ring" viewBox="0 0 100 100">
    <circle class="bg-ring" cx="50" cy="50" r="45"></circle>
    <circle class="fg-ring" id="fg-ring" cx="50" cy="50" r="45" stroke-dasharray="283" stroke-dashoffset="283"></circle>
  </svg>
  <div class="content" id="content">
    <svg class="heart-icon" id="heart" viewBox="0 0 24 24" fill="currentColor">
      <path d="M11.645 20.91l-.007-.003-.022-.012a15.247 15.247 0 01-.383-.218 25.18 25.18 0 01-4.244-3.17C4.688 15.36 2.25 12.174 2.25 8.25 2.25 5.322 4.714 3 7.688 3A5.5 5.5 0 0112 5.052 5.5 5.5 0 0116.313 3c2.973 0 5.437 2.322 5.437 5.25 0 3.925-2.438 7.111-4.739 9.256a25.175 25.175 0 01-4.244 3.17 15.247 15.247 0 01-.383.219l-.022.012-.007.004-.003.001a.752.752 0 01-.704 0l-.003-.001z"/>
    </svg>
    <div class="bpm-val" id="bpm">--</div>
    <div class="bpm-label">BPM</div>
  </div>
</div>
<script>
(function(){
  var widget = document.getElementById('widget'), content = document.getElementById('content'),
      bpmEl = document.getElementById('bpm'), heartEl = document.getElementById('heart'),
      fgRing = document.getElementById('fg-ring');
  var ws = null, reconnectTimer = null, staleTimer = null;
  var MIN_BPM = 60, MAX_BPM = 200, CIRCUMFERENCE = 2 * Math.PI * 45;
  
  function parseColor(hex) {
    if (!hex) return null;
    hex = hex.replace(/^#/, '');
    if (hex.length === 3) hex = hex.split('').map(function(c){return c+c;}).join('');
    if (hex.length !== 6) return null;
    var num = parseInt(hex, 16);
    if (isNaN(num)) return null;
    return { r: (num >> 16) & 255, g: (num >> 8) & 255, b: num & 255, gv: 0.6 };
  }
  
  var params = new URLSearchParams(window.location.search);
  var customColor = parseColor(params.get('color'));

  var zones = [
    { bpm: 0,   c: [220,180,180], g: 0.25 }, { bpm: 80,  c: [220,180,180], g: 0.25 },
    { bpm: 100, c: [120,220,160], g: 0.35 }, { bpm: 120, c: [250,204,21],  g: 0.4  },
    { bpm: 145, c: [249,115,22],  g: 0.55 }, { bpm: 170, c: [239,68,68],   g: 0.7  },
    { bpm: 200, c: [239,68,68],   g: 0.9  }
  ];
  function lerp(a,b,t){ return a+(b-a)*t; }
  function getZone(bpm){
    for(var i=0; i<zones.length-1; i++){
      if(bpm<=zones[i+1].bpm){
        var lo=zones[i], hi=zones[i+1], t=Math.max(0,Math.min(1,(bpm-lo.bpm)/(hi.bpm-lo.bpm)));
        return {
          r: Math.round(lerp(lo.c[0],hi.c[0],t)), g: Math.round(lerp(lo.c[1],hi.c[1],t)),
          b: Math.round(lerp(lo.c[2],hi.c[2],t)), gv: lerp(lo.g,hi.g,t)
        };
      }
    }
    var l=zones[zones.length-1]; return {r:l.c[0],g:l.c[1],b:l.c[2],gv:l.g};
  }
  
  function getWsUrl() {
    var p = new URLSearchParams(window.location.search);
    var h = p.get('host') || window.location.hostname;
    var po = p.get('port') || window.location.port;
    if (h === 'localhost') h = '127.0.0.1';
    return (window.location.protocol==='https:'?'wss:':'ws:')+'//'+h+(po?':'+po:'')+'/api/ws';
  }
  
  function update(bpm) {
    bpmEl.textContent = bpm; widget.classList.remove('disconnected'); content.classList.add('beating');
    var z = customColor || getZone(bpm);
    var rgb = 'rgb('+z.r+','+z.g+','+z.b+')', rgba = 'rgba('+z.r+','+z.g+','+z.b+','+z.gv.toFixed(2)+')';
    fgRing.style.color = rgb; fgRing.style.setProperty('--glow-color', rgba);
    heartEl.style.color = rgb; heartEl.style.setProperty('--glow-color', rgba);
    content.style.setProperty('--beat-dur', Math.round(60000/bpm)+'ms');
    
    var pct = Math.max(0, Math.min(1, (bpm - MIN_BPM)/(MAX_BPM - MIN_BPM)));
    fgRing.style.strokeDashoffset = CIRCUMFERENCE - (pct * CIRCUMFERENCE);
    
    clearTimeout(staleTimer); staleTimer = setTimeout(setDisconnected, 10000);
  }
  
  function setDisconnected() {
    widget.classList.add('disconnected'); content.classList.remove('beating'); bpmEl.textContent = '--';
    fgRing.style.strokeDashoffset = CIRCUMFERENCE;
  }
  
  function connect() {
    if(ws && (ws.readyState===1 || ws.readyState===0)) return;
    try { ws = new WebSocket(getWsUrl()); } catch(e) { return setTimeout(connect, 3000); }
    ws.onmessage = function(e){ try{ var d=JSON.parse(e.data); if(d.heart_rate) update(d.heart_rate); }catch(x){} };
    ws.onclose = function(){ setDisconnected(); setTimeout(connect, 3000); };
  }
  connect();
})();
</script>
</body>
</html>
"##;
