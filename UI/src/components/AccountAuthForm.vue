<script setup>
    import { computed } from 'vue';
    import { InfoFilled } from '@element-plus/icons-vue';

    /**
     * 属性定义
     * modelValue: 用于 v-model 绑定
     * customTips: 外部传入的自定义提示语
     */
    const props = defineProps({
        modelValue: {
            type: Object,
            required: true
        },
        customTips: {
            type: String,
            default: ''
        },
        small: {
            type: Boolean,
            default: false
        }
    });

    const emit = defineEmits(['update:modelValue']);

    // 使用计算属性来简化 v-model 的绑定逻辑
    const formData = computed({
        get: () => props.modelValue,
        set: (val) => emit('update:modelValue', val)
    });

    const defaultTips = '此凭据将用于 DLL 调起 WinLogon 认证，不会上传至任何云端。';
</script>

<template>
    <div class="account-auth-container">
        <el-form :model="formData" label-position="top" class="auth-form">
            <el-form-item label="账户类型">
                <el-select v-model="formData.accountType" placeholder="请选择账户类型" style="width: 100%">
                    <el-option label="本地账户 (Local Account)" value="local" />
                    <el-option label="联机账户 (Microsoft Account)" value="online" />
                </el-select>
            </el-form-item>

            <el-form-item :label="formData.accountType === 'local' ? 'Windows 用户名' : '微软账号 Email'">
                <el-input v-model="formData.username" :placeholder="formData.accountType === 'local' ? '例如: Administrator' : '例如: user@outlook.com'">
                    <template v-if="formData.accountType === 'local'" #prefix>
                        <span style="padding-left: 5px; color: #409EFF; font-weight: bold;">.\</span>
                    </template>
                </el-input>
            </el-form-item>

            <el-form-item label="系统登录密码">
                <template #label>
                    <div class="label-box">
                        <span>系统登录密码</span>
                        <el-tooltip
                            v-if="small"
                            :content="`<span>${customTips || defaultTips}</span>`"
                            placement="top-end"
                            raw-content
                        >
                            <el-icon class="question-icon"><WarnTriangleFilled /></el-icon>
                        </el-tooltip>
                    </div>
                </template>
                <el-input v-model="formData.password" type="password" show-password placeholder="请输入对应的账户密码，不支持Pin" />
            </el-form-item>

            <div class="auth-tips" v-if="!small">
                <el-icon>
                    <InfoFilled />
                </el-icon>
                <span v-html="customTips || defaultTips"></span>
            </div>
        </el-form>
    </div>
</template>

<style scoped>
    .account-auth-container {
        width: 100%;
    }

    .auth-form :deep(.el-form-item__label) {
        font-weight: 600;
        padding-bottom: 4px;
    }

    .auth-tips {
        margin-top: 15px;
        padding: 12px;
        background-color: #f4f4f5;
        border-left: 4px solid #909399;
        border-radius: 4px;
        font-size: 12px;
        color: #606266;
        display: flex;
        align-items: flex-start;
        gap: 8px;
        line-height: 1.6;
        margin-bottom: 15px;
    }

    .auth-tips .el-icon {
        margin-top: 2px;
        flex-shrink: 0;
    }

    .label-box{
        display: flex;
        align-items: center;
    }

    .question-icon{
        font-size: 18px;
        margin-left: 5px;
        cursor: pointer;
    }
</style>