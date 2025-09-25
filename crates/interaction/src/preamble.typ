#let adaptive-width(max-width: length, body) = {
	set page(width: max-width, height: auto, margin: 0pt)
	context {
		let content-width = measure(body).width
		if content-width < max-width {
			set page(width: content-width)
			body
		} else {
			body
		}
	}
}
#show: adaptive-width.with(max-width: 460pt)
#show: box.with(inset: 10pt)
