(() => {
  // Shared (Extension)/Resources/content.js
  var script = document.createElement("script");
  script.setAttribute("src", browser.runtime.getURL("nostr.build.js"));
  document.body.appendChild(script);
  window.addEventListener("message", async (message) => {
    const validEvents = [
      "getPubKey",
      "signEvent",
      "getRelays",
      "nip04.encrypt",
      "nip04.decrypt",
      "nip44.encrypt",
      "nip44.decrypt"
    ];
    let { kind, reqId, payload } = message.data;
    if (!validEvents.includes(kind)) return;
    payload = await browser.runtime.sendMessage({
      kind,
      payload,
      host: window.location.host
    });
    console.log(payload);
    kind = `return_${kind}`;
    window.postMessage({ kind, reqId, payload }, "*");
  });
})();
//# sourceMappingURL=data:application/json;base64,ewogICJ2ZXJzaW9uIjogMywKICAic291cmNlcyI6IFsiY29udGVudC5qcyJdLAogICJzb3VyY2VzQ29udGVudCI6IFsibGV0IHNjcmlwdCA9IGRvY3VtZW50LmNyZWF0ZUVsZW1lbnQoJ3NjcmlwdCcpO1xuc2NyaXB0LnNldEF0dHJpYnV0ZSgnc3JjJywgYnJvd3Nlci5ydW50aW1lLmdldFVSTCgnbm9zdHIuYnVpbGQuanMnKSk7XG5kb2N1bWVudC5ib2R5LmFwcGVuZENoaWxkKHNjcmlwdCk7XG5cbndpbmRvdy5hZGRFdmVudExpc3RlbmVyKCdtZXNzYWdlJywgYXN5bmMgbWVzc2FnZSA9PiB7XG4gICAgY29uc3QgdmFsaWRFdmVudHMgPSBbXG4gICAgICAgICdnZXRQdWJLZXknLFxuICAgICAgICAnc2lnbkV2ZW50JyxcbiAgICAgICAgJ2dldFJlbGF5cycsXG4gICAgICAgICduaXAwNC5lbmNyeXB0JyxcbiAgICAgICAgJ25pcDA0LmRlY3J5cHQnLFxuICAgICAgICAnbmlwNDQuZW5jcnlwdCcsXG4gICAgICAgICduaXA0NC5kZWNyeXB0JyxcbiAgICBdO1xuICAgIGxldCB7IGtpbmQsIHJlcUlkLCBwYXlsb2FkIH0gPSBtZXNzYWdlLmRhdGE7XG4gICAgaWYgKCF2YWxpZEV2ZW50cy5pbmNsdWRlcyhraW5kKSkgcmV0dXJuO1xuXG4gICAgcGF5bG9hZCA9IGF3YWl0IGJyb3dzZXIucnVudGltZS5zZW5kTWVzc2FnZSh7XG4gICAgICAgIGtpbmQsXG4gICAgICAgIHBheWxvYWQsXG4gICAgICAgIGhvc3Q6IHdpbmRvdy5sb2NhdGlvbi5ob3N0LFxuICAgIH0pO1xuICAgIGNvbnNvbGUubG9nKHBheWxvYWQpO1xuXG4gICAga2luZCA9IGByZXR1cm5fJHtraW5kfWA7XG5cbiAgICB3aW5kb3cucG9zdE1lc3NhZ2UoeyBraW5kLCByZXFJZCwgcGF5bG9hZCB9LCAnKicpO1xufSk7XG4iXSwKICAibWFwcGluZ3MiOiAiOztBQUFBLE1BQUksU0FBUyxTQUFTLGNBQWMsUUFBUTtBQUM1QyxTQUFPLGFBQWEsT0FBTyxRQUFRLFFBQVEsT0FBTyxnQkFBZ0IsQ0FBQztBQUNuRSxXQUFTLEtBQUssWUFBWSxNQUFNO0FBRWhDLFNBQU8saUJBQWlCLFdBQVcsT0FBTSxZQUFXO0FBQ2hELFVBQU0sY0FBYztBQUFBLE1BQ2hCO0FBQUEsTUFDQTtBQUFBLE1BQ0E7QUFBQSxNQUNBO0FBQUEsTUFDQTtBQUFBLE1BQ0E7QUFBQSxNQUNBO0FBQUEsSUFDSjtBQUNBLFFBQUksRUFBRSxNQUFNLE9BQU8sUUFBUSxJQUFJLFFBQVE7QUFDdkMsUUFBSSxDQUFDLFlBQVksU0FBUyxJQUFJLEVBQUc7QUFFakMsY0FBVSxNQUFNLFFBQVEsUUFBUSxZQUFZO0FBQUEsTUFDeEM7QUFBQSxNQUNBO0FBQUEsTUFDQSxNQUFNLE9BQU8sU0FBUztBQUFBLElBQzFCLENBQUM7QUFDRCxZQUFRLElBQUksT0FBTztBQUVuQixXQUFPLFVBQVUsSUFBSTtBQUVyQixXQUFPLFlBQVksRUFBRSxNQUFNLE9BQU8sUUFBUSxHQUFHLEdBQUc7QUFBQSxFQUNwRCxDQUFDOyIsCiAgIm5hbWVzIjogW10KfQo=
