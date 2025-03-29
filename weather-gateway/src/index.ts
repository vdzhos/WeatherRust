export default {
	async fetch(request: Request): Promise<Response> {
		// Only cache GET requests
		if (request.method !== "GET") {
			console.log("[Cache] Non-GET request received, bypassing cache:", request.url);
			return handleRequest(request);
		}

		// Use the default cache
		const cache = caches.default;
		// Use the full request URL as the cache key
		const cacheKey = new Request(request.url, request);

		// Try to find a cached response
		let response = await cache.match(cacheKey);
		if (response) {
			console.log("[Cache] Cache hit for:", request.url);
		} else {
			console.log("[Cache] Cache miss for:", request.url);
			// If not in cache, perform the request
			response = await handleRequest(request);
			// Read the response body as text (consumes the stream)
			const responseText = await response.text();

			// Create a new Response with fixed headers to ensure deterministic caching
			response = new Response(responseText, {
				status: response.status,
				headers: {
					"Content-Type": "application/json",
					"Access-Control-Allow-Origin": "*",
					// Cache the response for 60 seconds (adjust max-age as needed)
					"Cache-Control": "public, max-age=60",
				},
			});

			// Store a clone of the response in the cache
			await cache.put(cacheKey, response.clone());
			console.log("[Cache] Stored response in cache for:", request.url);
		}
		return response;
	},
};

// A helper function that handles the actual request forwarding
async function handleRequest(request: Request): Promise<Response> {
	const originalUrl = new URL(request.url);

	// Extract path and query from the incoming request
	const path = originalUrl.pathname; // e.g., /VisualCrossingWebServices/rest/services/timeline/Kyiv
	const query = originalUrl.search;    // includes the ?

	// Inject API key if not present in the query
	const apiKey = "4WWQEDQX2PLN5W97DGKEDXAS3";
	const hasApiKey = query.includes("key=");
	const newQuery = hasApiKey ? query : `${query ? query + "&" : "?"}key=${apiKey}`;

	// Build the new target URL
	const targetUrl = `https://weather.visualcrossing.com${path}${newQuery}`;
	console.log("[Fetch] Fetching target URL:", targetUrl);

	// Forward the request to the external API
	const response = await fetch(targetUrl, {
		method: request.method,
		headers: {
			"User-Agent": "IC-Gateway",
		},
	});

	// Copy the response body and return a new Response with deterministic headers
	const body = await response.text();
	return new Response(body, {
		status: response.status,
		headers: {
			"Content-Type": "application/json",
			"Access-Control-Allow-Origin": "*",
			"Cache-Control": "public, max-age=60",
		},
	});
}
