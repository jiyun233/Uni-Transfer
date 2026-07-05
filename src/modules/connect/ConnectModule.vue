<script setup lang="ts">
import { NText } from "naive-ui"
import { ref, onMounted } from "vue"
import { invoke } from "@tauri-apps/api/core"

const ips = ref<Record<string, string>>({})

async function getAllIPv4Addresses() {
    try {
        const result = await invoke<Record<string, string>>('get_all_ipv4')
        ips.value = result
    } catch (error) {
        console.error('获取IP地址失败:', error)
    }
}

onMounted(() => {
    getAllIPv4Addresses()
})
</script>

<template>
    <div class="connect-module">
        <NText tag="h2" strong>可用于局域网连接的 IPv4 地址</NText>

        <div v-if="Object.keys(ips).length > 0" class="ip-list">
            <div v-for="(ip, name) in ips" :key="name" class="ip-item">
                <NText depth="2">{{ name }}：</NText>
                <NText strong style="color: #2ecc71; font-size: 18px;">{{ ip }}</NText>
            </div>
        </div>

        <NText v-else depth="3" style="color: #e74c3c;">未检测到可用网络连接</NText>
    </div>
</template>

<style scoped>
.connect-module {
    height: 100%;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 20px;
    padding: 24px;
    text-align: center;
}

.ip-list {
    display: flex;
    flex-direction: column;
    gap: 12px;
}

.ip-item {
    background: rgba(255, 255, 255, 0.05);
    padding: 10px 16px;
    border-radius: 8px;
    min-width: 280px;
}
</style>