'use strict';
const status = document.getElementById('status');
const localButton = document.getElementById('start-local');
const remoteForm = document.getElementById('remote-form');
const session = document.getElementById('session');
const disconnectButton = document.getElementById('disconnect');
const proof = document.getElementById('proof');
const proofPanel = document.getElementById('setup-proof');
let active = false;
let busy = false;

function render() {
  localButton.disabled = busy || active;
  remoteForm.querySelector('button').disabled = busy || active;
  remoteForm.querySelector('input').disabled = busy || active;
  disconnectButton.disabled = busy;
  session.hidden = !active;
}

async function invoke(command, args = {}) {
  if (!window.__TAURI__?.core?.invoke) throw new Error('请在构建后的 RstrBot 桌面应用中打开此页面。');
  return window.__TAURI__.core.invoke(command, args);
}

async function connect(command, args) {
  busy = true;
  status.textContent = command === 'start_local' ? '正在启动本机后端…' : '正在打开远程服务器…';
  render();
  try {
    const result = await invoke(command, args);
    active = true;
    document.getElementById('session-description').textContent = result.mode === 'local'
      ? `本机服务：${result.url}\n数据位置：${result.data_dir}`
      : `远程服务器：${result.url}`;
    disconnectButton.textContent = result.mode === 'local' ? '断开并停止本机进程' : '断开远程工作区';
    proof.value = result.setup_proof ?? '';
    proofPanel.hidden = !result.setup_proof;
    status.textContent = result.mode === 'remote'
      ? '已打开服务器页面。连接和登录状态以该页面的实际响应为准。'
      : '后端已确认开始监听。请在工作区窗口继续设置或登录。';
  } catch (error) {
    status.textContent = String(error);
  } finally {
    busy = false;
    render();
  }
}

localButton.addEventListener('click', () => connect('start_local'));
remoteForm.addEventListener('submit', (event) => {
  event.preventDefault();
  if (remoteForm.reportValidity()) connect('connect_remote', { server: document.getElementById('server').value.trim() });
});
disconnectButton.addEventListener('click', async () => {
  busy = true;
  status.textContent = '正在关闭工作区…';
  render();
  try {
    await invoke('disconnect');
    active = false;
    proof.value = '';
    proofPanel.hidden = true;
    status.textContent = '已断开，已有数据保留。';
  } catch (error) {
    status.textContent = String(error);
  } finally { busy = false; render(); }
});
document.getElementById('select-proof').addEventListener('click', () => { proof.focus(); proof.select(); });
render();

