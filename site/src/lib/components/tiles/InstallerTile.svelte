<script lang="ts">
	import { Icon } from '@facile/muse';
	import { Stage, Tile } from '@facile/mise-en-scene';
	import { t } from '$lib/i18n';
	import { glyphs, marks } from '$lib/marks';
</script>

<Tile title={t('surface.install.title')} body={t('surface.install.body')}>
	<Stage width={520} height={400} compact={{ below: 400, width: 340, height: 380 }}>
		{#snippet children({ compact })}
			<div class="screen flux-pal" class:compact>
				<Icon icon={marks.flux} size={34} />
				<div class="welcome">{t('demo.install.welcome')}</div>
				<div class="line">{t('demo.install.line')}</div>
				<div class="choices">
					<div class="choice">
						<Icon icon={glyphs.usb} size={20} />
						<span class="name">{t('demo.install.try')}</span>
						<span class="sub">{t('demo.install.try.line')}</span>
					</div>
					<div class="choice main">
						<Icon icon={glyphs.disk} size={20} />
						<span class="name">{t('demo.install.install')}</span>
						<span class="sub">{t('demo.install.install.line')}</span>
					</div>
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
		align-items: flex-start;
		justify-content: center;
		padding: 0 34px;
		overflow: hidden;
		border-radius: 16px;
		background: var(--fx-p);
		box-shadow: 0 18px 40px -24px rgb(0 0 0 / 0.35);
	}
	.screen.compact {
		inset: 14px 14px 6px;
		padding: 0 20px;
	}
	.welcome {
		margin-top: 16px;
		font-family: var(--font-fc-title);
		font-size: 24px;
		font-weight: 600;
		letter-spacing: -0.01em;
	}
	.line {
		margin-top: 4px;
		color: var(--fx-m);
		font-size: 12px;
	}
	.choices {
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: 10px;
		width: 100%;
		margin-top: 20px;
	}
	.choice {
		display: flex;
		flex-direction: column;
		gap: 4px;
		min-height: 120px;
		padding: 14px;
		border-radius: 12px;
		background: var(--fx-c);
	}
	.choice.main {
		background: var(--fx-f);
		color: var(--fx-p);
	}
	.name {
		margin-top: 8px;
		font-family: var(--font-fc-title);
		font-size: 14px;
		font-weight: 600;
	}
	.sub {
		color: var(--fx-m);
		font-size: 11px;
		line-height: 1.35;
	}
	.main .sub {
		color: inherit;
		opacity: 0.7;
	}
</style>
