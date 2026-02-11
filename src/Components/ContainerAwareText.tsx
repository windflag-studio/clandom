import React, { useState, useEffect, useRef, CSSProperties, useCallback } from 'react';

interface ContainerAwareTextProps {
    /** 子文本内容 */
    children: React.ReactNode;
    /** 最小字号（像素） */
    minFontSize?: number;
    /** 最大字号（像素） */
    maxFontSize?: number;
    /** 目标比例（文字整体宽度占父容器宽度的百分比） */
    scaleRatio?: number;
    /** 监听哪个父级元素（默认直接父元素） */
    parentSelector?: string;
    /** 额外样式 */
    style?: CSSProperties;
    /** 类名 */
    className?: string;
    /** 是否启用自适应 */
    enabled?: boolean;
    /** 测量方法：根据宽度适应还是根据高度适应 */
    fitMode?: 'width' | 'height' | 'both';
    /** 防抖延迟（毫秒） */
    debounceDelay?: number;
    /** 是否显示调试信息（开发环境下） */
    showDebug?: boolean;
}

// 防抖函数
const debounce = <T extends (...args: any[]) => any>(
    func: T,
    wait: number
): ((...args: Parameters<T>) => void) => {
    let timeout: NodeJS.Timeout | null = null;
    return (...args: Parameters<T>) => {
        if (timeout) clearTimeout(timeout);
        timeout = setTimeout(() => func(...args), wait);
    };
};

// 获取文本内容的工具函数
const getTextContent = (node: React.ReactNode): string => {
    if (typeof node === 'string' || typeof node === 'number') {
        return node.toString();
    }
    if (Array.isArray(node)) {
        return node.map(getTextContent).join('');
    }
    if (React.isValidElement(node) && node.props) {
        return getTextContent(node.props.children);
    }
    return '';
};

const ContainerAwareText: React.FC<ContainerAwareTextProps> = ({
    children,
    minFontSize = 12,
    maxFontSize = 48,
    scaleRatio = 0.8,
    parentSelector,
    style,
    className = '',
    enabled = true,
    fitMode = 'width',
    debounceDelay = 100,
    showDebug = process.env.NODE_ENV === 'development',
}) => {
    const textRef = useRef<HTMLSpanElement>(null);
    const [fontSize, setFontSize] = useState<number>(minFontSize);
    const [parentElement, setParentElement] = useState<HTMLElement | null>(null);
    const textContent = getTextContent(children);

    // 测量函数
    const measureAndAdjust = useCallback(() => {
        if (!enabled || !textRef.current || !textContent.trim()) return;

        const element = textRef.current;
        let parent: HTMLElement | null = null;

        if (parentSelector) {
            parent = element.closest(parentSelector);
        } else {
            parent = element.parentElement;
        }

        if (!parent) {
            console.warn('未找到父元素');
            return;
        }

        setParentElement(parent);

        // 创建一个临时元素进行测量，避免影响当前渲染
        const tempElement = document.createElement('span');
        tempElement.style.cssText = `
            position: absolute;
            visibility: hidden;
            white-space: nowrap;
            font-family: inherit;
            font-weight: inherit;
            font-style: inherit;
            letter-spacing: inherit;
        `;
        
        // 设置最大字号作为测量基准
        tempElement.style.fontSize = `${maxFontSize}px`;
        tempElement.textContent = textContent;
        
        // 插入到DOM中获取计算样式
        document.body.appendChild(tempElement);
        
        // 获取文本的实际测量尺寸
        const textMetrics = tempElement.getBoundingClientRect();
        const parentRect = parent.getBoundingClientRect();
        
        // 清理临时元素
        document.body.removeChild(tempElement);
        
        let calculatedSize = maxFontSize;
        
        if (fitMode === 'width' || fitMode === 'both') {
            const textWidth = textMetrics.width;
            const parentWidth = parentRect.width;
            
            if (textWidth > 0 && parentWidth > 0) {
                // 计算当前比例
                const currentRatio = textWidth / parentWidth;
                // 调整到目标比例
                calculatedSize = maxFontSize * (scaleRatio / currentRatio);
            }
        }
        
        if (fitMode === 'height' || fitMode === 'both') {
            const textHeight = textMetrics.height;
            const parentHeight = parentRect.height;
            
            if (textHeight > 0 && parentHeight > 0) {
                const currentRatio = textHeight / parentHeight;
                calculatedSize = Math.min(calculatedSize, maxFontSize * (scaleRatio / currentRatio));
            }
        }
        
        // 限制在最小和最大字号之间
        calculatedSize = Math.max(minFontSize, Math.min(maxFontSize, calculatedSize));
        
        // 设置最终字号
        setFontSize(Math.round(calculatedSize));
    }, [minFontSize, maxFontSize, scaleRatio, parentSelector, enabled, textContent, fitMode]);

    useEffect(() => {
        if (!enabled) return;

        const debouncedMeasure = debounce(measureAndAdjust, debounceDelay);
        
        // 初始测量
        debouncedMeasure();
        
        // 设置MutationObserver监听文本内容变化
        const observer = new MutationObserver(debouncedMeasure);
        if (textRef.current) {
            observer.observe(textRef.current, {
                characterData: true,
                childList: true,
                subtree: true
            });
        }
        
        // 监听父元素尺寸变化
        let resizeObserver: ResizeObserver | null = null;
        if (textRef.current) {
            const element = textRef.current;
            let parent: HTMLElement | null = null;
            
            if (parentSelector) {
                parent = element.closest(parentSelector);
            } else {
                parent = element.parentElement;
            }
            
            if (parent) {
                resizeObserver = new ResizeObserver(debouncedMeasure);
                resizeObserver.observe(parent);
            }
        }
        
        // 监听窗口变化
        window.addEventListener('resize', debouncedMeasure);
        
        return () => {
            observer.disconnect();
            if (resizeObserver) {
                resizeObserver.disconnect();
            }
            window.removeEventListener('resize', debouncedMeasure);
        };
    }, [measureAndAdjust, enabled, parentSelector, debounceDelay]);

    return (
        <span
            ref={textRef}
            className={`container-aware-text ${className}`}
            style={{
                fontSize: `${fontSize}px`,
                lineHeight: 1.2,
                display: 'inline-flex',
                flexDirection: 'column',
                alignItems: 'flex-start',
                justifyContent: 'center',
                transition: 'font-size 0.3s ease',
                whiteSpace: 'nowrap',
                verticalAlign: 'middle',
                ...style,
            }}
        >
            <span style={{ whiteSpace: 'nowrap' }}>
                {children}
            </span>
            {showDebug && (
                <span 
                    style={{ 
                        display: 'block',
                        fontSize: '10px', 
                        opacity: 0.6,
                        marginTop: '2px',
                        lineHeight: 1.3,
                        whiteSpace: 'normal',
                        fontFamily: 'monospace, Consolas, "Courier New"',
                        background: 'rgba(0,0,0,0.05)',
                        padding: '2px 4px',
                        borderRadius: '2px',
                        wordBreak: 'break-all',
                        width: '100%',
                        boxSizing: 'border-box'
                    }}
                    title="调试信息"
                >
                    父容器: {parentElement?.offsetWidth || 0}px × {parentElement?.offsetHeight || 0}px<br />
                    当前字号: {Math.round(fontSize)}px<br />
                    目标比例: {scaleRatio} ({Math.round(scaleRatio * 100)}%)<br />
                    文本内容: "{textContent.length > 20 ? textContent.substring(0, 20) + '...' : textContent}"
                </span>
            )}
        </span>
    );
};

export default ContainerAwareText;