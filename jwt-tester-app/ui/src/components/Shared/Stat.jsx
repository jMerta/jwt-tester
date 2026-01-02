import React from "react";

export function Stat({ label, value }) {
    return (
        <div className="stat">
            <span>{label}</span>
            <strong>{value}</strong>
        </div>
    );
}
