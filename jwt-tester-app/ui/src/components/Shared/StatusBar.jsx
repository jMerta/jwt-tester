import React, { useEffect } from "react";

export function StatusBar({ status, setStatus }) {
    // Auto-clear status after 3 seconds
    useEffect(() => {
        if (status) {
            const timer = setTimeout(() => {
                setStatus("");
            }, 3000);
            return () => clearTimeout(timer);
        }
    }, [status, setStatus]);

    if (!status) return null;

    return <div className="status-bar">{status}</div>;
}
