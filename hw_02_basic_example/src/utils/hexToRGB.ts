/**
 * 十六进制字符串转数字颜色
 * @param hex
 * @param alpha
 */
const hexToRGB = (hex: string, alpha: number) => {
    const r = parseInt(hex.slice(1, 3), 16);
    const g = parseInt(hex.slice(3, 5), 16);
    const b = parseInt(hex.slice(5, 7), 16);

    return `rgba(${r},${g},${b},${alpha})`;
};

export default hexToRGB;