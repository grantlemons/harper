import logoSvg from '../logo.svg';
import { linter } from './lint';
import { Plugin, addIcon, Menu } from 'obsidian';
import { LocalLinter, WorkerLinter } from 'harper.js';
import { HarperSettingTab } from './HarperSettingTab';

function suggestionToLabel(sug) {
	if (sug.kind() == 'Remove') {
		return 'Remove';
	} else {
		return `Replace with "${sug.get_replacement_text()}"`;
	}
}

function initHarperInstance(useWebWorker) {
	if (useWebWorker) {
		console.log('Switching to `WorkerLinter`');
		harper = new WorkerLinter();
	} else {
		console.log('Switching to `LocalLinter`');
		harper = new LocalLinter();
	}
	harper.setup();
}

let harper;

initHarperInstance(true);

const harperLinter = (plugin) =>
	linter(
		async (view) => {
			if (!plugin.shouldLint()) {
				return [];
			}

			const text = view.state.doc.sliceString(-1);

			const lints = await harper.lint(text);

			return lints.map((lint) => {
				let span = lint.span();

				return {
					from: span.start,
					to: span.end,
					severity: 'error',
					title: lint.lint_kind(),
					message: lint.message(),
					actions: lint.suggestions().map((sug) => {
						return {
							name: suggestionToLabel(sug),
							apply: (view) => {
								if (sug === 'Remove') {
									view.dispatch({
										changes: {
											from: span.start,
											to: span.end,
											insert: ''
										}
									});
								} else {
									view.dispatch({
										changes: {
											from: span.start,
											to: span.end,
											insert: sug.get_replacement_text()
										}
									});
								}
							}
						};
					})
				};
			});
		},
		{
			delay: -1,
			needsRefresh: () => {
				let temp = plugin.lintSettingModified;
				plugin.lintSettingModified = false;
				return temp;
			}
		}
	);

export default class HarperPlugin extends Plugin {
	/** @private */
	shouldAutoLint = true;
	/** @public */
	lintSettingModified = false;

	/** @public
	 * @returns {Promise<Record<string, string>>} */
	async getDescriptions() {
		return await harper.getLintDescriptions();
	}

	/** @public
	 * @returns {Promise<Record<string, any>>} */
	async getSettings() {
		this.lintSettingChanged();

		let lintSettings = await harper.getLintConfig();

		return { ...this.settings, lintSettings };
	}

	/** @public
	 * @param {Record<string, any>} settings
	 * @returns {Promise<void>} */
	async setSettings(settings) {
		if (settings == null) {
			settings = {};
		}

		if (settings.useWebWorker == undefined) {
			settings.useWebWorker = true;
		}

		if (settings.lintSettings == undefined) {
			settings.lintSettings = {};
		}

		if (settings.lintSettings.spell_check == undefined) {
			settings.lintSettings.spell_check = false;
		}

		await harper.setLintConfig(settings.lintSettings);
		this.lintSettingChanged();
		this.saveData(settings);

		if (this.settings?.useWebWorker != settings.useWebWorker) {
			initHarperInstance(settings.useWebWorker);
		}

		this.settings = settings;
	}

	async onload() {
		console.log(harperLinter(this));

		this.registerEditorExtension([harperLinter(this)]);
		this.app.workspace.updateOptions();

		addIcon('harper-logo', logoSvg);

		this.setupCommands();
		this.setupStatusBar();

		await this.setSettings(await this.loadData());
		this.addSettingTab(new HarperSettingTab(this.app, this));
	}

	setupCommands() {
		this.addCommand({
			id: 'harper-toggle-auto-lint',
			name: 'Toggle automatic grammar checking',
			callback: () => this.toggleAutoLint()
		});
	}

	setupStatusBar() {
		/** @type HTMLElement */
		let statusBarItem = this.addStatusBarItem();
		statusBarItem.className += ' mod-clickable';

		let button = document.createElement('span');
		button.style = 'width:24px';
		button.innerHTML = logoSvg;

		button.addEventListener('click', (event) => {
			const menu = new Menu();

			menu.addItem((item) =>
				item
					.setTitle(`${this.shouldAutoLint ? 'Disable' : 'Enable'} automatic checking`)
					.setIcon('documents')
					.onClick(() => {
						this.toggleAutoLint();
					})
			);

			menu.showAtMouseEvent(event);
		});

		statusBarItem.appendChild(button);
	}

	shouldLint() {
		return this.shouldAutoLint;
	}

	/** @param {boolean} shouldAutoLint  */
	setAutoLint(shouldAutoLint) {
		this.shouldAutoLint = shouldAutoLint;
		this.lintSettingChanged();
	}

	toggleAutoLint() {
		this.shouldAutoLint = !this.shouldAutoLint;
		this.lintSettingChanged();
	}

	lintSettingChanged() {
		this.lintSettingModified = true;
		this.app.workspace.updateOptions();
	}
}
