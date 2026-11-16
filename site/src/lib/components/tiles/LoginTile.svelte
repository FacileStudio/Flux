<script lang="ts">
	import { Icon } from '@facile/muse';
	import { Stage, Tile } from '@facile/mise-en-scene';
	import { t } from '$lib/i18n';
	import { glyphs, marks } from '$lib/marks';
</script>

<Tile title={t('surface.login.title')} body={t('surface.login.body')}>
	<Stage width={520} height={400} compact={{ below: 400, width: 340, height: 380 }}>
		{#snippet children({ compact })}
			<div class="screen flux-pal" class:compact>
				<div class="veil"></div>
				<div class="time">09:41</div>
				<div class="date">{t('demo.login.date')}</div>
				<div class="card">
					<Icon icon={marks.flux} size={26} />
					<div class="welcome">{t('demo.login.welcome')}</div>
					<div class="user">Flux</div>
					<div class="field"><span class="dots">••••••••</span><Icon icon={glyphs.eye} size={14} /></div>
					<div class="actions"><span class="login">{t('demo.login.button')}</span></div>
				</div>
				<div class="power">
					<span class="round"><Icon icon={glyphs.restart} size={14} /></span>
					<span class="round"><Icon icon={glyphs.power} size={14} /></span>
				</div>
			</div>
		{/snippet}
	</Stage>
</Tile>

<style>
	.screen {
		position: absolute;
		inset: 24px 24px 8px;
		display: flex;
		flex-direction: column;
		align-items: center;
		overflow: hidden;
		border-radius: 16px;
		background: var(--flux-wallpaper);
	}
	.screen.compact {
		inset: 14px 14px 6px;
	}
	.veil {
		position: absolute;
		inset: 0;
		background: color-mix(in oklab, var(--fx-p) 52%, transparent);
	}
	.time,
	.date,
	.card,
	.power {
		position: relative;
	}
	.time {
		margin-top: 22px;
		font-family: var(--font-fc-title);
		font-size: 40px;
		font-weight: 300;
		line-height: 1;
		letter-spacing: -0.02em;
		font-variant-numeric: tabular-nums;
	}
	.date {
		margin-top: 6px;
		font-size: 12px;
		font-weight: 500;
	}
	.card {
		display: flex;
		flex-direction: column;
		align-items: center;
		width: 260px;
		margin-top: 18px;
		padding: 16px 18px 14px;
		border-radius: 16px;
		background: var(--fx-p);
		box-shadow: 0 24px 50px -24px rgb(0 0 0 / 0.5);
	}
	.welcome {
		margin-top: 8px;
		font-family: var(--font-fc-title);
		font-size: 15px;
		font-weight: 600;
	}
	.user {
		align-self: flex-start;
		margin-top: 10px;
		color: var(--fx-m);
		font-size: 11px;
	}
	.field {
		display: flex;
		align-items: center;
		justify-content: space-between;
		width: 100%;
		height: 30px;
		margin-top: 4px;
		padding: 0 12px;
		border-radius: 9px;
		background: var(--fx-s);
		color: var(--fx-m);
	}
	.dots {
		color: var(--fx-f);
		letter-spacing: 0.1em;
	}
	.actions {
		display: flex;
		width: 100%;
		margin-top: 8px;
	}
	.login {
		flex: 1;
		padding: 7px 0;
		border-radius: 9px;
		background: var(--fx-f);
		color: var(--fx-p);
		font-size: 12px;
		font-weight: 600;
		text-align: center;
	}
	.power {
		display: flex;
		gap: 10px;
		margin-top: auto;
		margin-bottom: 14px;
	}
	.round {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		width: 30px;
		height: 30px;
		border-radius: 999px;
		background: color-mix(in oklab, var(--fx-p) 80%, transparent);
	}
</style>
