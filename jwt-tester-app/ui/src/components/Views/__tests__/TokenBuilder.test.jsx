import React from "react";
import { render, screen, waitFor, fireEvent, act } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { vi, describe, it, expect, beforeEach, afterEach } from "vitest";
import { TokenBuilder } from "../TokenBuilder.jsx";

const mockFetch = (response) => {
  global.fetch = vi.fn().mockResolvedValue(response);
};

describe("TokenBuilder", () => {
  beforeEach(() => {
    mockFetch({
      ok: true,
      json: async () => ({ ok: true, data: { token: "header.payload.sig" } }),
    });
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  it("sends claims as a string and time fields as strings", async () => {
    const user = userEvent.setup();
    const setStatus = vi.fn();
    const onRefresh = vi.fn();

    render(
      <TokenBuilder
        projectName="alpha"
        keys={[{ id: "key-1", name: "Primary" }]}
        onRefresh={onRefresh}
        setStatus={setStatus}
      />
    );

    const claimsField = screen.getByLabelText(/JSON Payload/i);
    await act(async () => {
      fireEvent.change(screen.getByLabelText(/Algorithm/i), {
        target: { value: "rs256" },
      });
      fireEvent.change(screen.getByLabelText(/Signing Key/i), {
        target: { value: "key-1" },
      });
      fireEvent.change(screen.getByLabelText(/iss \(Issuer\)/i), {
        target: { value: "issuer" },
      });
      fireEvent.change(screen.getByLabelText(/sub \(Subject\)/i), {
        target: { value: "subject" },
      });
      fireEvent.change(screen.getByLabelText(/aud \(Audience\)/i), {
        target: { value: "api, mobile" },
      });
      fireEvent.change(screen.getByLabelText(/jti \(JWT ID\)/i), {
        target: { value: "jwt-id" },
      });
      fireEvent.change(claimsField, {
        target: { value: '{ "sub": "123" }' },
      });
      fireEvent.change(screen.getByLabelText(/iat \(Issued At\)/i), {
        target: { value: "1700000000" },
      });
      fireEvent.change(screen.getByLabelText(/nbf \(Not Before\)/i), {
        target: { value: "1700000100" },
      });
      fireEvent.change(screen.getByLabelText(/exp \(Expiration\)/i), {
        target: { value: "1700000200" },
      });
      await user.click(screen.getByRole("button", { name: /Generate Token/i }));
    });

    await waitFor(() => expect(global.fetch).toHaveBeenCalled());

    const encodeCall = global.fetch.mock.calls.find(
      ([url]) => url === "/api/jwt/encode"
    );
    expect(encodeCall).toBeTruthy();
    const body = JSON.parse(encodeCall[1].body);
    expect(body.key_id).toBe("key-1");
    expect(body.alg).toBe("rs256");
    expect(body.iss).toBe("issuer");
    expect(body.sub).toBe("subject");
    expect(body.aud).toEqual(["api", "mobile"]);
    expect(body.jti).toBe("jwt-id");
    expect(typeof body.claims).toBe("string");
    expect(body.claims).toContain('"sub"');
    expect(body.iat).toBe("1700000000");
    expect(body.nbf).toBe("1700000100");
    expect(body.exp).toBe("1700000200");
    await waitFor(() => expect(setStatus).toHaveBeenCalled());
  });

  it("blocks invalid JSON claims before sending", async () => {
    const user = userEvent.setup();
    const setStatus = vi.fn();

    render(
      <TokenBuilder
        projectName="alpha"
        keys={[]}
        onRefresh={vi.fn()}
        setStatus={setStatus}
      />
    );

    const claimsField = screen.getByLabelText(/JSON Payload/i);
    await act(async () => {
      fireEvent.change(claimsField, { target: { value: "not-json" } });
      await user.click(
        screen.getByRole("button", { name: /Generate Token/i })
      );
    });

    await waitFor(() =>
      expect(setStatus).toHaveBeenCalledWith("Invalid JSON in Custom Claims.")
    );
    expect(global.fetch).not.toHaveBeenCalled();
  });
});
