(() => {
	var t = localStorage.getItem("moltis-theme") || "system";
	if (t === "system") t = matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light";
	document.documentElement.setAttribute("data-theme", t);
	document.documentElement.style.background = t === "dark" ? "#0f1115" : "#fafafa";
	document.documentElement.style.color = t === "dark" ? "#e0e0e0" : "#222";
})();
