import { exists as existsFile, readFile, remove as removeFile, readTextFile, readTextFileLines } from '@tauri-apps/plugin-fs';
import { executableDir } from '@tauri-apps/api/path';
import { invoke } from '@tauri-apps/api/core';
import { formatObjectString } from '../utils/function';
import { error as errorLog } from '@tauri-apps/plugin-log';
export function useFile() {
    let cachedBaseDir = null;
    const resolveBaseDir = async () => {
        if (cachedBaseDir) {
            return cachedBaseDir;
        }
        let baseDir = localStorage.getItem("exe_dir");
        if (!baseDir || baseDir === "null") {
            try {
                const res = await invoke('get_exe_dir');
                if (res && res.data && res.data.path) {
                    baseDir = res.data.path;
                }
            } catch (error) {
                errorLog(formatObjectString("获取程序目录失败：", error));
            }
        }
        if (!baseDir || baseDir === "null") {
            baseDir = await executableDir();
        }
        localStorage.setItem("exe_dir", baseDir);
        if (!baseDir.endsWith("\\") && !baseDir.endsWith("/")) {
            baseDir += "\\";
        }
        cachedBaseDir = baseDir;
        return baseDir;
    };
    /**
     * 判断文件是否存在
     * @param {String} path 文件路径
     * @returns {Promise<Boolean>}
     */
    const exists = async (path) => {
        const baseDir = await resolveBaseDir();
        return existsFile(baseDir + path);
    };

    /**
     * 删除文件
     * @param {String} path 文件路径
     * @returns {Promise<void>}
     */
    const reomve = async (path) => {
        const baseDir = await resolveBaseDir();
        return removeFile(baseDir + path);
    };

    /**
     * 读取图片文件并转换为指定格式（base64/blob）
     * @param {String} path 文件路径
     * @param {('base64' | 'blob')} [type='base64'] 输出类型，可选 base64/blob
     * @param {('jpg' | 'png' | 'jpeg')} [format='jpg'] 图片格式，默认jpg，支持png/jpg/jpeg
     * @returns {Promise<string | Blob>} 对应类型的图片数据，base64返回字符串，blob返回Blob对象
     */
    const read = async (path, type = 'base64', format = 'jpg') => {
        const baseDir = await resolveBaseDir();
        const fullPath = baseDir + path;
        return new Promise((resolve, reject) => {
            existsFile(fullPath).then(()=>{
                // 读取文件
                return readFile(fullPath);
            }).then((fileData)=>{
                // 判断输出类型
                if (type === 'base64') {
                    // Uint8Array 转成 二进制字符串
                    let binaryStr = '';
                    for (let i = 0; i < fileData.length; i++) {
                        binaryStr += String.fromCharCode(fileData[i]);
                    }
                    // 二进制字符串 转成 Base64
                    const base64Str = btoa(binaryStr);
                    const mimeType = format === 'jpg' || format === 'jpeg' 
                        ? 'image/jpeg' 
                        : 'image/png';
                    resolve(`data:${mimeType};base64,${base64Str}`);
                } else {
                    // blob 可以直接转
                    const mimeType = format === 'jpg' || format === 'jpeg' 
                        ? 'image/jpeg' 
                        : 'image/png';
                    resolve(new Blob([fileData], { type: mimeType }));
                }
            }).catch((error)=>{
                const info = formatObjectString("文件读取失败：", error);
                errorLog(info);
                reject(info);
            })
        });
    }

    /**
     * 把文件读取成txt并返回
     * @param {String} path 文件路径
     * @returns {Promise<string>} 文本数据
     */
    const readText = async (path) => {
        const baseDir = await resolveBaseDir();
        const fullPath = baseDir + path;
        return new Promise((resolve, reject) => {
            existsFile(fullPath).then(()=>{
                // 读取文件
                return readTextFile(fullPath);
            }).then((data)=>{
                resolve(data);
            }).catch((error)=>{
                const info = formatObjectString("文件读取失败：", error);
                errorLog(info);
                reject(info);
            })
        });
    }

    return {
        exists,
        read,
        reomve,
        readText
    };
}
