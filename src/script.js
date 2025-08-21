if (document.readyState === "complete") {
	onReady();
} else {
	window.addEventListener(
		"load",
		() => {
			onReady();
		},
		false,
		true,
	);
}
function onReady() {
	document
		.getElementsByClassName("mhy-hoyolab-account-block")[0]
		.dispatchEvent(new Event("click"));
}
