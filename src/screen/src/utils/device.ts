/**
 * 判断当前平台是否为 macOS
 * @returns boolean - 如果是 macOS 返回 true，否则返回 false
 */
export const isMacOS = (): boolean => {
  if (typeof window === 'undefined') return false;
  
  // 使用更现代的 navigator.userAgentData.platform
  if ((window.navigator as any).userAgentData?.platform) {
    return (window.navigator as any).userAgentData.platform.toLowerCase() === 'macos';
  }
  
  // 回退方案：使用 navigator.userAgent
  const userAgent = window.navigator.userAgent.toLowerCase();
  return userAgent.includes('macintosh') || userAgent.includes('mac os x');
};

/**
 * 判断当前平台是否为 Windows
 * @returns boolean - 如果是 Windows 返回 true，否则返回 false
 */
export const isWindows = (): boolean => {
  if (typeof window === 'undefined') return false;
  
  // 使用更现代的 navigator.userAgentData.platform
  if ((window.navigator as any).userAgentData?.platform) {
    return (window.navigator as any).userAgentData.platform.toLowerCase() === 'windows';
  }
  
  // 回退方案：使用 navigator.userAgent
  const userAgent = window.navigator.userAgent.toLowerCase();
  return userAgent.includes('windows');
};

/**
 * 获取当前平台类型
 * @returns 'macos' | 'windows' | 'other' - 返回当前平台类型
 */
export const getPlatform = (): 'macos' | 'windows' | 'other' => {
  if (isMacOS()) return 'macos';
  if (isWindows()) return 'windows';
  return 'other';
};
