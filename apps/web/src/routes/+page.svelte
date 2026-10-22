<svelte:head><title>dibibest — schemas, both ways</title></svelte:head>

<main>
	<h1>dibibest</h1>
	<p class="lede">
		Convert database schemas between DBML, SQL, Prisma and an interactive ERD — and see exactly what
		survived the round trip.
	</p>
	<a class="cta" href="/app">Open the workspace →</a>
	<p class="fine">Runs entirely in your browser. No account, no upload.</p>
</main>

<style>
	main {
		max-width: 640px;
		margin: 0 auto;
		padding: 18vh var(--sp-4) var(--sp-4);
	}
	h1 {
		font-size: 40px;
		letter-spacing: -0.02em;
	}
	.lede {
		margin: var(--sp-4) 0 var(--sp-4);
		color: var(--text-2);
		font-size: 18px;
		line-height: 1.5;
	}
	.cta {
		display: inline-block;
		background: var(--accent-strong);
		color: #fff;
		font-weight: 600;
		padding: 10px 20px;
		border-radius: var(--r-md);
		text-decoration: none;
	}
	.fine {
		margin-top: var(--sp-4);
		color: var(--text-3);
		font-size: 13px;
	}
</style>
