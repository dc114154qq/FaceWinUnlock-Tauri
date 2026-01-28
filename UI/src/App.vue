<script setup>
	import { ref } from 'vue';
	import { RouterView } from 'vue-router';
	import { connect } from './utils/sqlite.js';
	import { formatObjectString } from './utils/function.js';
	import { ElMessageBox } from 'element-plus';
	import { useOptionsStore } from "./stores/options";
	import { useRouter } from 'vue-router';
	import { invoke } from '@tauri-apps/api/core';
	import { info, warn } from '@tauri-apps/plugin-log';
	import { useFacesStore } from './stores/faces.js';
	import { attachConsole } from "@tauri-apps/plugin-log";
	import { executableDir } from '@tauri-apps/api/path';
	import { getVersion } from '@tauri-apps/api/app';
	import { getCurrentWindow } from '@tauri-apps/api/window';

	const isInit = ref(false);
	const router = useRouter();
	const optionsStore = useOptionsStore();
	const facesStore = useFacesStore();
	const currentWindow = getCurrentWindow();

	// 打包时注释
	attachConsole();

	const resolveExeDir = async () => {
		try {
			const res = await invoke('get_exe_dir');
			if (res && res.data && res.data.path) {
				return res.data.path;
			}
		} catch (error) {
			warn(formatObjectString('获取程序目录失败，尝试前端路径API：', error));
		}
		return await executableDir();
	};

	const runStep = async (label, action) => {
		try {
			return await action();
		} catch (error) {
			throw new Error(`${label}失败：${formatObjectString(error)}`);
		}
	};

	const initApp = async () => {
		try {
			const exeDir = await runStep('获取程序目录', resolveExeDir);
			localStorage.setItem('exe_dir', exeDir);
			await runStep('连接数据库', connect);
			await runStep('加载配置', () => optionsStore.init());
			await runStep('初始化模型', () => invoke('init_model'));
			await runStep('加载面容数据', () => facesStore.init());

			let is_initialized = optionsStore.getOptionByKey('is_initialized');
			if(is_initialized.index == -1 || is_initialized.data.val != 'true'){
				warn("程序未初始化，强制跳转初始化界面");
				router.push('/init');
			}
			info("程序初始化完成");
			if(optionsStore.getOptionValueByKey('silentRun') != "true"){
				currentWindow.isVisible().then((visible) => {
					if(!visible){
						currentWindow.show();
					}
					currentWindow.setFocus();
				}).catch((error)=>{
					warn(formatObjectString("获取窗口状态失败 ",error));
				})
			}
			isInit.value = true;
		} catch (error) {
			ElMessageBox.alert(formatObjectString(error), '程序初始化失败', {
				confirmButtonText: '确定',
				callback: () => {
					invoke("close_app");
				}
			});
		}
	};

	initApp();

	// 版本号不影响运行，不用放在上面
	getVersion().then((v)=>{
		localStorage.setItem('version', v);
	});
</script>

<template>
	<div class="app-wrapper" v-if="isInit">
		<router-view />
    </div>
</template>

<style scoped>
	.app-wrapper {
		height: 100vh;
		width: 100vw;
	}
</style>
