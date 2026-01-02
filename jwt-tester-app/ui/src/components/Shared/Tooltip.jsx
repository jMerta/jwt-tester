import React, { useState, useRef, useEffect } from 'react';
import { createPortal } from 'react-dom';
import './Tooltip.css';

export function Tooltip({ text, children }) {
    const [isVisible, setIsVisible] = useState(false);
    const [position, setPosition] = useState({ top: 0, left: 0 });
    const triggerRef = useRef(null);

    useEffect(() => {
        if (isVisible && triggerRef.current) {
            const rect = triggerRef.current.getBoundingClientRect();
            setPosition({
                top: rect.top + window.scrollY - 10, // 10px spacing above
                left: rect.left + rect.width / 2
            });
        }
    }, [isVisible]);

    return (
        <span
            className="tooltip-trigger"
            ref={triggerRef}
            onMouseEnter={() => setIsVisible(true)}
            onMouseLeave={() => setIsVisible(false)}
        >
            {children}
            {isVisible && createPortal(
                <div
                    className="tooltip-bubble"
                    style={{
                        top: `${position.top}px`,
                        left: `${position.left}px`
                    }}
                >
                    {text}
                </div>,
                document.body
            )}
        </span>
    );
}
