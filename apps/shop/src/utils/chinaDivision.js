// 使用 jsDelivr CDN 获取省市区数据 (约 180KB Gzipped)
// 来源: https://github.com/modood/Administrative-divisions-of-China
const DATA_URL = 'https://unpkg.com/china-division/dist/pcas-code.json';

let addressCache = null;

export async function getAddressData() {
    if (addressCache) return addressCache;
    
    try {
        const response = await fetch(DATA_URL);
        if (!response.ok) throw new Error('Failed to load address data');
        const data = await response.json();
        addressCache = data;
        return data;
    } catch (error) {
        console.error('Address data load failed:', error);
        return [];
    }
}