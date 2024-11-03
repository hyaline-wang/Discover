<template>
    <div>
        <el-table v-loading="loading" :data="devicesList" style="width: 100%;height: 97vh;"
            :row-class-name="rowClassName">
            <!-- <el-table v-loading="loading" :data="wifiList" style="width: 100%;height: 84vh;" :row-class-name="rowClassName"> -->
            <el-table-column prop="device_name" label="DEV NAME" width="180">
            </el-table-column>
            <el-table-column label="IP-ADDR" width="200">
                <template #default="scope">
                    <div v-for="(ip,inter) in scope.row.ip_addresses">
                        {{ inter }}: {{ ip }} 
                    </div>
                </template>
            </el-table-column>
            <el-table-column label="状态" width="180">
                <template #default="scope">
                    <el-icon size="22" v-if="getOnlineStatus(scope.row) == 'Online'" color="green">
                        <SuccessFilled />
                    </el-icon>
                    <el-icon size="22" v-else-if="getOnlineStatus(scope.row) == 'Timeout'" color="#e36e40">
                        <WarningFilled />
                    </el-icon>    
                    <el-icon size="22" v-else color="red">
                        <CircleCloseFilled />
                    </el-icon>               
                </template>
            </el-table-column>
            <el-table-column label="操作" width="200">
                <template #default="scope">
                    <el-button  v-if="getOnlineStatus(scope.row) == 'Online'" @click="openWebPage(scope.row)" type="primary">连接</el-button>
                    <div v-else>不可用</div>
                </template>
            </el-table-column>
        </el-table>
    </div>
</template>
<script setup>
import { ref, onMounted } from 'vue';
import { SuccessFilled,WarningFilled,CircleCloseFilled } from '@element-plus/icons-vue'
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import { listen } from '@tauri-apps/api/event';
import { invoke } from "@tauri-apps/api/core";
import { WebviewWindow } from '@tauri-apps/api/webviewWindow'
import axios from 'axios';
// const emit = defineEmits(['sta_connect']);

const devicesList = ref([]);
const loading = ref(false);
const error = ref(null);
function handleSubmit() {
    // 执行提交操作（可选）
    ready_to_connect.value = false; // 关闭对话框
}
function getOnlineStatus(device){
    const now = Math.floor(Date.now() / 1000); // 获取当前时间的 UNIX 时间戳（秒）
    console.log(now - device.last_updated);
    if(now - device.last_updated < 10) {
        return 'Online'
    }
    else if (now - device.last_updated < 30) {
        return 'Timeout'
    }
    else {
        return 'Offline'
    }
}

const appWebview = getCurrentWebviewWindow();
appWebview.listen('json-event', (event) => {
    console.log("event.payload");
    console.log(event.payload);
    const devices = JSON.parse(event.payload); // 解析 JSON 字符串
    console.log(devices);
});
let intervalId;

const getDevices = async () => {
    const dev_str = await invoke("get_devices");
    devicesList.value = JSON.parse(dev_str); // 解析 JSON 字符串
    console.log(devicesList.value);

};

onMounted(() => {
    // 每 5 秒更新 currentTime
    intervalId = setInterval(getDevices, 5000);
    getDevices(); // 立即获取一次问候消息

});

function openWebPage(device) {
    const window_name = device.device_name;
  // 创建一个新的窗口实例
  const newWindow = new WebviewWindow('newWindow', {
    // url: 'http://emnavi.tech', // 新窗口的页面，可以是本地文件或远程URL
    url: 'http://localhost:5173/login',
    title: window_name,    // 窗口标题
    width: 600,             // 窗口宽度
    height: 400,            // 窗口高度
    resizable: true         // 是否可调整窗口大小
  });
  console.log(newWindow);
  newWindow.once('tauri://created', function () {
    // webview successfully created
    console.log("新窗口已创建并显示");

  });
  newWindow.once('tauri://error', function (e) {
    // an error happened creating the webview
  });
};
// const unlisten = await listen('json-event', (event) => {
//         console.log(event.payload);

// });
// unlisten();

const ready_to_connect = ref(false);
const rowClassName = ({ row }) => {
    console.log(row);
    console.log(row.active == 'yes' ? 'success-row' : '');
    return row.active == 'yes' ? 'success-row' : ''; // 如果 active 不是 'yes'，则返回高亮类
};

const selectedWifi = ref(null);
// const openWebPage = (device) => {

//     selectedWifi.value = wifi; // 选中当前 WiFi
//     ready_to_connect.value = true; // 打开对话框
// };
const fetchWifiList = async () => {
    // emit("sta_connect"); // 发出提交事件
    loading.value = true;
    error.value = null;
    console.log("appWebview");

    try {
        const response = await axios.get('/api/wifi'); // 替换为你的 API 地址
        wifiList.value = response.data; // 假设返回的数据是一个 WiFi 对象数组
    } catch (err) {
        error.value = '无法获取 WiFi 列表';
        console.error(err);
    } finally {
        loading.value = false;
    }
};

// 在组件挂载时获取 WiFi 列表
// onMounted(fetchWifiList);
</script>

<style>
.el-table .success-row {
    /* --el-table-tr-bg-color: var(--el-color-warning-light-9); */
    --el-table-tr-bg-color: #45553d96;
}
</style>