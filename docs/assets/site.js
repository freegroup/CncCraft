/* Suche und Seitenleiste der CncCraft-Hilfe. Läuft ohne Server: der Index liegt als
   Skriptdatei vor (assets/search-index.js), daher geht es auch aus dem Dateisystem. */
(function () {
  "use strict";
  var index = window.SEARCH_INDEX || [];
  var root = document.body.getAttribute("data-help-root") || "./";

  function norm(s) {
    return s.toLowerCase().replace(/ä/g, "ae").replace(/ö/g, "oe").replace(/ü/g, "ue").replace(/ß/g, "ss");
  }
  var prepared = index.map(function (e) {
    return { e: e, t: norm(e.t), x: norm(e.x), c: norm(e.c) };
  });

  function esc(s) {
    return s.replace(/[&<>"]/g, function (c) { return { "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;" }[c]; });
  }

  function search(q) {
    var words = norm(q).split(/\s+/).filter(Boolean);
    if (!words.length) return [];
    var out = [];
    prepared.forEach(function (p) {
      var score = 0;
      for (var i = 0; i < words.length; i++) {
        var w = words[i], s = 0;
        if (p.t.indexOf(w) === 0) s += 12;
        else if (p.t.indexOf(w) > 0) s += 8;
        if (p.c.indexOf(w) >= 0) s += 2;
        var n = 0, at = p.x.indexOf(w);
        while (at >= 0 && n < 5) { n++; at = p.x.indexOf(w, at + w.length); }
        s += n;
        if (!s) return;          // jedes Wort muss vorkommen
        score += s;
      }
      out.push({ e: p.e, score: score });
    });
    out.sort(function (a, b) { return b.score - a.score; });
    return out.slice(0, 12);
  }

  function snippet(text, q) {
    var w = norm(q).split(/\s+/).filter(Boolean)[0] || "";
    var at = norm(text).indexOf(w);
    var from = Math.max(0, at - 40);
    var part = text.substr(from, 130);
    var html = esc(part);
    if (w) {
      try { html = html.replace(new RegExp("(" + w.replace(/[.*+?^${}()|[\]\\]/g, "\\$&") + ")", "ig"), "<mark>$1</mark>"); } catch (e) {}
    }
    return (from > 0 ? "… " : "") + html + (from + 130 < text.length ? " …" : "");
  }

  document.querySelectorAll(".suche").forEach(function (box) {
    var input = box.querySelector("input");
    var list = box.querySelector(".treffer");
    var active = -1;
    function render() {
      var q = input.value.trim();
      if (q.length < 2) { box.classList.remove("offen"); return; }
      var res = search(q);
      if (!res.length) {
        list.innerHTML = '<div class="leer">Keine Treffer für „' + esc(q) + '“.</div>';
      } else {
        list.innerHTML = res.map(function (r) {
          return '<a href="' + root + r.e.u + '"><div class="k">' + esc(r.e.c) + '</div><div class="t">' + esc(r.e.t) +
            (r.e.n ? ' <span class="pille">Neu</span>' : "") + '</div><div class="s">' + snippet(r.e.x, q) + "</div></a>";
        }).join("");
      }
      active = -1;
      box.classList.add("offen");
    }
    input.addEventListener("input", render);
    input.addEventListener("focus", render);
    input.addEventListener("keydown", function (ev) {
      var items = list.querySelectorAll("a");
      if (ev.key === "ArrowDown" || ev.key === "ArrowUp") {
        ev.preventDefault();
        if (!items.length) return;
        active = (active + (ev.key === "ArrowDown" ? 1 : -1) + items.length) % items.length;
        items.forEach(function (a, i) { a.classList.toggle("aktiv", i === active); });
        items[active].scrollIntoView({ block: "nearest" });
      } else if (ev.key === "Enter" && items.length) {
        window.location.href = items[Math.max(active, 0)].getAttribute("href");
      } else if (ev.key === "Escape") {
        box.classList.remove("offen"); input.blur();
      }
    });
    document.addEventListener("click", function (ev) { if (!box.contains(ev.target)) box.classList.remove("offen"); });
  });

  // "/" springt ins Suchfeld
  document.addEventListener("keydown", function (ev) {
    if (ev.key === "/" && !/input|textarea/i.test((document.activeElement || {}).tagName || "")) {
      var i = document.querySelector(".suche input");
      if (i) { ev.preventDefault(); i.focus(); }
    }
  });

  var knopf = document.querySelector(".menue-knopf");
  var leiste = document.querySelector(".seitenleiste");
  if (knopf && leiste) knopf.addEventListener("click", function () { leiste.classList.toggle("offen"); });

  var cur = document.querySelector(".seitenleiste a[aria-current]");
  if (cur && cur.scrollIntoView) { var bar = cur.closest(".seitenleiste"); if (bar) bar.scrollTop = cur.offsetTop - bar.clientHeight / 3; }
})();
