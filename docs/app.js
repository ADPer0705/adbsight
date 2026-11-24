(() => {
  const TUI_SCREENS = {
    "1": `<span class="c-cyan">╭─ Device Identity & Security ─────────────────╮╭─ CPU & Load ─────────────────────────────────╮</span>
<span class="c-muted">│</span> Model:   Google Pixel 9 Pro (caiman)         <span class="c-muted">││</span> <span class="c-green">Total: 27.4%</span>  •  Cores: 8  •  Load: 1.42 1.28  <span class="c-muted">│</span>
<span class="c-muted">│</span> Serial:  192.168.1.42:41893 (<span class="c-cyan">Wi-Fi</span>)          <span class="c-muted">││</span> <span class="c-cyan">▁▂▃▄▆▅▃▂▄▆▇▅▃▄▆▅▄▃▂▃▄▅▆▄▃▂▃▄▅▃▂▃▄▅▆▅▄▃▂▃▄▅▃▂</span> <span class="c-muted">│</span>
<span class="c-muted">│</span> Android: 15 (SDK 35) • Patch 2026-11-05      <span class="c-muted">││</span> C0: 31%  C1: 24%  C2: 40%  C3: 18%             <span class="c-muted">│</span>
<span class="c-muted">│</span> SELinux: <span class="c-green">Enforcing</span>   Verified Boot: green     <span class="c-muted">││</span> C4: 29%  C5: 22%  C6: 35%  C7: 20%             <span class="c-muted">│</span>
<span class="c-cyan">╰──────────────────────────────────────────────╯╰──────────────────────────────────────────────╯</span>
<span class="c-cyan">╭─ Memory & Swap ──────────────────────────────╮╭─ Battery & Thermals ─────────────────────────╮</span>
<span class="c-muted">│</span> <span class="c-green">███████████████░░░░░░░░░░░</span> 56.1% (8.77/15.6G)<span class="c-muted">││</span> <span class="c-green">█████████████████████░░░░</span> 87% Discharging 32°C <span class="c-muted">│</span>
<span class="c-muted">│</span> <span class="c-cyan">███░░░░░░░░░░░░░░░░░░░░░░░</span> Swap 13.7%        <span class="c-muted">││</span> Health: Good • 4120 mV • -385 mA               <span class="c-muted">│</span>
<span class="c-muted">│</span> Avail: 6.86 GiB  Cached: 4.62 GiB            <span class="c-muted">││</span> Thermals: soc_big: 41.2°C | gpu: 38.6°C      <span class="c-muted">│</span>
<span class="c-cyan">╰──────────────────────────────────────────────╯╰──────────────────────────────────────────────╯</span>
<span class="c-cyan">╭─ Storage Mounts ─────────────────────────────╮╭─ Network & Throughput ───────────────────────╮</span>
<span class="c-muted">│</span> /data              <span class="c-green">41%</span>  93.8 / 228.9 GiB     <span class="c-muted">││</span> Wi-Fi: Lab-5GHz-Mesh • 192.168.1.42 • -51dBm <span class="c-muted">│</span>
<span class="c-muted">│</span> /storage/emulated  <span class="c-green">41%</span>  93.8 / 228.9 GiB     <span class="c-muted">││</span> Rate:  RX 54.2 KiB/s   TX 18.4 KiB/s         <span class="c-muted">│</span>
<span class="c-muted">│</span> /system            <span class="c-red">94%</span>  4.30 / 4.58 GiB      <span class="c-muted">││</span> <span class="c-green">▂▃▅▃▂▄▆▇▅▃▂▃▄▆▅▃▂▄▅▇▆▄▃▂▃▄▅▆▅▃▂▃▄▅▃▂▃▄▆▅▄▃▂</span>   <span class="c-muted">│</span>
<span class="c-cyan">╰──────────────────────────────────────────────╯╰──────────────────────────────────────────────╯</span>`,

    "2": `<span class="c-cyan">╭─ System & Hardware [Tab to switch] ──────────────────────────────────────────────────────────╮</span>
<span class="c-muted">│</span>  <span class="c-cyan">Properties</span>   │   Settings   │   Sensors & HW   │   Dumpsys                                    <span class="c-muted">│</span>
<span class="c-cyan">╰──────────────────────────────────────────────────────────────────────────────────────────────╯</span>
<span class="c-cyan">╭─ Filter [/ to search, Esc to clear] ─────────────────────────────────────────────────────────╮</span>
<span class="c-muted">│</span> &gt; ro.build                                                                                   <span class="c-muted">│</span>
<span class="c-cyan">╰──────────────────────────────────────────────────────────────────────────────────────────────╯</span>
<span class="c-cyan">╭─ System Properties (16) ─────────────────────────────────────────────────────────────────────╮</span>
<span class="c-muted">│</span> <span class="c-cyan">PROPERTY KEY</span>                          <span class="c-cyan">VALUE</span>                                                  <span class="c-muted">│</span>
<span class="c-muted">│</span> ro.boot.verifiedbootstate             green                                                  <span class="c-muted">│</span>
<span class="c-muted">│</span> ro.build.id                           AP3A.241105.008                                        <span class="c-muted">│</span>
<span class="c-muted">│</span> ro.build.version.release              15                                                     <span class="c-muted">│</span>
<span class="c-muted">│</span> ro.build.version.sdk                  35                                                     <span class="c-muted">│</span>
<span class="c-muted">│</span> ro.build.version.security_patch       2026-11-05                                             <span class="c-muted">│</span>
<span class="c-muted">│</span> persist.adb.wifi.guid                 adb-pixel9-wireless                                    <span class="c-muted">│</span>
<span class="c-cyan">╰──────────────────────────────────────────────────────────────────────────────────────────────╯</span>`,

    "3": `<span class="c-cyan">╭─ Processes & Services ───────────────────────────────────────────────────────────────────────╮</span>
<span class="c-muted">│</span> View: Processes [Tab] • Sort: <span class="c-amber">CPU%</span> [s cycle] • Filter: "" [/] • [K] Kill PID • [r] Refresh   <span class="c-muted">│</span>
<span class="c-cyan">╰──────────────────────────────────────────────────────────────────────────────────────────────╯</span>
<span class="c-cyan">╭─ Processes (184) ────────────────────────────────────────────────────────────────────────────╮</span>
<span class="c-muted">│</span> <span class="c-cyan">PID     USER         CPU%     RSS       S  NAME</span>                                              <span class="c-muted">│</span>
<span class="c-muted">│</span> 1842    system       <span class="c-green">9.4%</span>     305.1 MiB S  system_server                                     <span class="c-muted">│</span>
<span class="c-muted">│</span> 1104    system       <span class="c-green">5.2%</span>     125.0 MiB S  /system/bin/surfaceflinger                        <span class="c-muted">│</span>
<span class="c-muted">│</span> 2490    u0_a185      <span class="c-green">4.1%</span>     239.4 MiB S  com.android.systemui                              <span class="c-muted">│</span>
<span class="c-muted">│</span> 5120    u0_a241      <span class="c-green">3.8%</span>     193.7 MiB R  com.android.chrome                                <span class="c-muted">│</span>
<span class="c-muted">│</span> 3320    u0_a142      <span class="c-green">2.1%</span>     160.9 MiB S  com.google.android.gms.persistent                 <span class="c-muted">│</span>
<span class="c-muted">│</span> 892     shell        <span class="c-green">0.9%</span>      13.9 MiB S  /apex/com.android.adbd/bin/adbd                 <span class="c-muted">│</span>
<span class="c-cyan">╰──────────────────────────────────────────────────────────────────────────────────────────────╯</span>`,

    "4": `<span class="c-cyan">╭─ Packages & Apps ────────────────────────────────────────────────────────────────────────────╮</span>
<span class="c-muted">│</span> Scope: All [f] • Filter: "" [/] • [Enter] Inspect  [l] Launch  [x] Stop  [p] Pull APK        <span class="c-muted">│</span>
<span class="c-cyan">╰──────────────────────────────────────────────────────────────────────────────────────────────╯</span>
<span class="c-cyan">╭─ Installed (214) ──────────────────────────╮╭─ Package Deep Inspector ───────────────────────╮</span>
<span class="c-muted">│</span> <span class="c-green">USR</span>  com.android.chrome           10241    <span class="c-muted">││</span> Package: com.termux                            <span class="c-muted">│</span>
<span class="c-muted">│</span> <span class="c-muted">SYS</span>  com.android.settings         1000     <span class="c-muted">││</span> Version: 0.118.0 (code 118)                    <span class="c-muted">│</span>
<span class="c-muted">│</span> <span class="c-green">USR</span>  org.thoughtcrime.securesms   10288    <span class="c-muted">││</span> SDK:     min 24 / target 28                    <span class="c-muted">│</span>
<span class="c-muted">│</span> <span class="c-amber">USR  com.termux                   10310</span>    <span class="c-muted">││</span> Granted Permissions (4):                       <span class="c-muted">│</span>
<span class="c-muted">│</span> <span class="c-green">USR</span>  org.mozilla.firefox          10294    <span class="c-muted">││</span>   + android.permission.INTERNET                <span class="c-muted">│</span>
<span class="c-muted">│</span>                                            <span class="c-muted">││</span>   + android.permission.ACCESS_NETWORK_STATE    <span class="c-muted">│</span>
<span class="c-cyan">╰────────────────────────────────────────────╯╰────────────────────────────────────────────────╯</span>`,

    "5": `<span class="c-cyan">╭─ Live Logcat Stream ─────────────────────────────────────────────────────────────────────────╮</span>
<span class="c-muted">│</span> <span class="c-green">LIVE</span>  •  Min Level: V [v/d/i/w/e/f]  •  Filter: "" [/]  •  [Space] Pause  [C] Clear  [E] Export<span class="c-muted">│</span>
<span class="c-cyan">╰──────────────────────────────────────────────────────────────────────────────────────────────╯</span>
<span class="c-cyan">╭─ Logcat Buffer (2140 matching / 2140 stored) ────────────────────────────────────────────────╮</span>
<span class="c-muted">│</span> 11-24 17:30:01.104  1842  <span class="c-green"> I </span> <span class="c-cyan">ActivityManager:</span> Displayed com.termux/.HomeActivity: +84ms      <span class="c-muted">│</span>
<span class="c-muted">│</span> 11-24 17:30:01.942   892  <span class="c-cyan"> D </span> <span class="c-cyan">AdbdWifi:</span> TLS keepalive ACK from 192.168.1.10:5037           <span class="c-muted">│</span>
<span class="c-muted">│</span> 11-24 17:30:02.410  2210  <span class="c-amber"> W </span> <span class="c-cyan">ThermalEngine:</span> soc_big crossed 43.0C soft threshold           <span class="c-muted">│</span>
<span class="c-muted">│</span> 11-24 17:30:03.005  4412  <span class="c-red"> E </span> <span class="c-cyan">MediaProvider:</span> Skipped hidden .nomedia cache entry             <span class="c-muted">│</span>
<span class="c-muted">│</span> 11-24 17:30:03.851  1842  <span class="c-green"> I </span> <span class="c-cyan">NetworkMonitor:</span> Validated wlan0 IPv4/IPv6 dual-stack          <span class="c-muted">│</span>
<span class="c-cyan">╰──────────────────────────────────────────────────────────────────────────────────────────────╯</span>`,

    "6": `<span class="c-cyan">╭─ Filesystem Explorer ────────────────────────────────────────────────────────────────────────╮</span>
<span class="c-muted">│</span> Path: /sdcard  •  [Enter] Open/Preview  [Backspace] Up  [d] /sdcard  [p] Pull  [u] Push      <span class="c-muted">│</span>
<span class="c-cyan">╰──────────────────────────────────────────────────────────────────────────────────────────────╯</span>
<span class="c-cyan">╭─ Entries (6) ──────────────────────────────╮╭─ Preview: /sdcard/device_notes.txt ──────────╮</span>
<span class="c-muted">│</span> <span class="c-cyan">DIR</span>   Download              -   drwxrwx--- <span class="c-muted">││</span> # ADBSight Device Notes                      <span class="c-muted">│</span>
<span class="c-muted">│</span> <span class="c-cyan">DIR</span>   DCIM                  -   drwxrwx--- <span class="c-muted">││</span> Wi-Fi Debugging: Active on 192.168.1.42:41893<span class="c-muted">│</span>
<span class="c-muted">│</span> <span class="c-amber">LNK</span>   sdcard -> /storage..  21B lrwxrwxrwx <span class="c-muted">││</span> SELinux: Enforcing                             <span class="c-muted">│</span>
<span class="c-muted">│</span> FILE  device_notes.txt    256B  -rw-rw---- <span class="c-muted">││</span>                                                <span class="c-muted">│</span>
<span class="c-muted">│</span> FILE  crash_trace.log     1.0K  -rw-rw---- <span class="c-muted">││</span>                                                <span class="c-muted">│</span>
<span class="c-cyan">╰────────────────────────────────────────────╯╰────────────────────────────────────────────────╯</span>`,

    "7": `<span class="c-cyan">╭─ Personal Data Inspector [Tab switch, S snapshot, e export] ─────────────────────────────────╮</span>
<span class="c-muted">│</span>   Contacts   │   <span class="c-cyan">SMS</span>   │   Call Logs   │   Accounts                                          <span class="c-muted">│</span>
<span class="c-cyan">╰──────────────────────────────────────────────────────────────────────────────────────────────╯</span>
<span class="c-cyan">╭─ SMS Messages (2) ───────────────────────────────────────────────────────────────────────────╮</span>
<span class="c-muted">│</span> <span class="c-cyan">TYPE     ADDRESS        DATE                 BODY</span>                                            <span class="c-muted">│</span>
<span class="c-muted">│</span> <span class="c-amber">Inbox</span>    +1-555-0142    2026-11-24 15:20:00  Hey, the Wi-Fi ADB pairing QR code worked!      <span class="c-muted">│</span>
<span class="c-muted">│</span> <span class="c-amber">Sent</span>     +1-555-0175    2026-11-24 13:06:40  Sent the kernel trace &amp; APK dump to /sdcard.    <span class="c-muted">│</span>
<span class="c-cyan">╰──────────────────────────────────────────────────────────────────────────────────────────────╯</span>`,

    "w": `<span class="c-cyan">╭─ Wireless Debugging &amp; Device Manager [Tab switch, Esc close] ────────────────────────────────╮</span>
<span class="c-muted">│</span>  1 Devices &amp; mDNS   │   <span class="c-cyan">2 QR Code Pair (Android 11+)</span>   │   3 Pair Code / Direct IP:Port      <span class="c-muted">│</span>
<span class="c-cyan">├──────────────────────────────────────────────┬───────────────────────────────────────────────┤</span>
<span class="c-muted">│</span> ██████████████████████████████████████       <span class="c-muted">│</span> <span class="c-cyan">Android 11+ Wireless QR Pairing</span>               <span class="c-muted">│</span>
<span class="c-muted">│</span> ██ ▄▄▄▄▄ █▀▄ ▄█▄▀▄▄█ ▄▄▄▄▄ ██                <span class="c-muted">│</span> 1. Settings -> Developer options              <span class="c-muted">│</span>
<span class="c-muted">│</span> ██ █   █ █ ▀█▀▄▄█▄▀█ █   █ ██                <span class="c-muted">│</span> 2. Enable Wireless debugging                  <span class="c-muted">│</span>
<span class="c-muted">│</span> ██ █▄▄▄█ █▀▄█▄▀ ▀▄▀█ █▄▄▄█ ██                <span class="c-muted">│</span> 3. Tap 'Pair device with QR code'             <span class="c-muted">│</span>
<span class="c-muted">│</span> ██▄▄▄▄▄▄▄█▄█ █▄█▄█▄█▄▄▄▄▄▄▄██                <span class="c-muted">│</span> Service:  adbsight-8K2mPq9xRt                 <span class="c-muted">│</span>
<span class="c-muted">│</span> ██ ▀▄ ▄▀▄█▀▄█▀▄█▀▄ ▄█▄▀▄▀▄██                <span class="c-muted">│</span> Password: ************                        <span class="c-muted">│</span>
<span class="c-muted">│</span> ██████████████████████████████████████       <span class="c-muted">│</span> <span class="c-green">Status: Listening via mDNS for pairing...</span>     <span class="c-muted">│</span>
<span class="c-cyan">╰──────────────────────────────────────────────┴───────────────────────────────────────────────╯</span>`
  };

  // Interactive TUI Tab Switcher
  const screenEl = document.getElementById("tui-screen");
  const tabBtns = document.querySelectorAll(".tui-tab");
  tabBtns.forEach((btn) => {
    btn.addEventListener("click", () => {
      tabBtns.forEach((b) => b.classList.remove("active"));
      btn.classList.add("active");
      const key = btn.getAttribute("data-tab");
      if (screenEl && TUI_SCREENS[key]) {
        screenEl.innerHTML = TUI_SCREENS[key];
      }
    });
  });

  // Copy buttons for code blocks
  document.querySelectorAll(".copy-btn").forEach((btn) => {
    btn.addEventListener("click", async () => {
      const wrap = btn.closest(".code-wrap");
      const code = wrap?.querySelector("pre")?.innerText || "";
      try {
        await navigator.clipboard.writeText(code);
        const orig = btn.textContent;
        btn.textContent = "Copied ✓";
        setTimeout(() => {
          btn.textContent = orig;
        }, 1500);
      } catch {
        btn.textContent = "Copy failed";
      }
    });
  });

  // Wiki Search Index
  const searchInput = document.getElementById("wiki-search");
  const resultsBox = document.getElementById("search-results");
  const sections = Array.from(document.querySelectorAll(".wiki-section"));
  const indexItems = sections.map((sec) => ({
    id: sec.id,
    title: sec.querySelector("h1, h2")?.textContent?.replace("┌─", "").trim() || sec.id,
    tag: sec.querySelector(".section-tag")?.textContent?.trim() || "Section",
    text: sec.textContent.toLowerCase(),
  }));

  function runSearch(query) {
    const q = query.trim().toLowerCase();
    if (!q) {
      resultsBox.classList.remove("open");
      resultsBox.innerHTML = "";
      return;
    }
    const matches = indexItems.filter(
      (item) => item.title.toLowerCase().includes(q) || item.text.includes(q)
    );
    if (matches.length === 0) {
      resultsBox.innerHTML = `<div class="search-result-item"><span class="search-result-sub">No matching wiki sections for "${query}"</span></div>`;
    } else {
      resultsBox.innerHTML = matches
        .slice(0, 8)
        .map(
          (m) =>
            `<a class="search-result-item" href="#${m.id}"><span class="search-result-title">${m.title}</span><span class="search-result-sub">${m.tag}</span></a>`
        )
        .join("");
    }
    resultsBox.classList.add("open");
  }

  searchInput?.addEventListener("input", (e) => runSearch(e.target.value));
  resultsBox?.addEventListener("click", () => {
    resultsBox.classList.remove("open");
    closeSidebar();
  });

  document.addEventListener("keydown", (e) => {
    if (e.key === "/" && document.activeElement !== searchInput) {
      e.preventDefault();
      searchInput?.focus();
    } else if (e.key === "Escape") {
      resultsBox?.classList.remove("open");
      searchInput?.blur();
    }
  });

  // Mobile sidebar toggle
  const sidebar = document.getElementById("sidebar");
  const menuBtn = document.getElementById("menu-btn");
  function closeSidebar() {
    sidebar?.classList.remove("open");
  }
  menuBtn?.addEventListener("click", () => {
    sidebar?.classList.toggle("open");
  });

  // Active nav highlighting on scroll
  const navLinks = Array.from(document.querySelectorAll(".nav-link"));
  navLinks.forEach((l) => l.addEventListener("click", closeSidebar));

  const observer = new IntersectionObserver(
    (entries) => {
      for (const entry of entries) {
        if (entry.isIntersecting) {
          const id = entry.target.id;
          navLinks.forEach((link) => {
            link.classList.toggle("active", link.getAttribute("href") === `#${id}`);
          });
        }
      }
    },
    { rootMargin: "-20% 0px -65% 0px" }
  );
  sections.forEach((sec) => observer.observe(sec));
})();
