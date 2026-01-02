import React from "react";
import { render, screen, waitFor, act, fireEvent } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { vi, describe, it, expect, beforeEach, afterEach } from "vitest";
import { TokenVerifier } from "../TokenVerifier.jsx";

const b64Url = (obj) =>
  btoa(JSON.stringify(obj))
    .replace(/\+/g, "-")
    .replace(/\//g, "_")
    .replace(/=+$/g, "");

const makeToken = (payload) => {
  const header = b64Url({ alg: "HS256", typ: "JWT" });
  const body = b64Url(payload);
  return `${header}.${body}.sig`;
};

const mockFetch = (response) => {
  global.fetch = vi.fn().mockResolvedValue(response);
};

describe("TokenVerifier", () => {
  beforeEach(() => {
    mockFetch({
      ok: true,
      json: async () => ({ ok: true, data: { valid: true } }),
    });
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  it("verifies even when exp is missing by default", async () => {
    const user = userEvent.setup();
    const setStatus = vi.fn();

    render(
      <TokenVerifier
        projectName="alpha"
        keys={[]}
        setStatus={setStatus}
      />
    );

    await act(async () => {
      const tokenField = screen.getByLabelText(/Token to Verify/i);
      await user.type(tokenField, makeToken({ sub: "user" }), {
        parseSpecialCharSequences: false,
      });
      await user.click(screen.getByRole("button", { name: /Verify Token/i }));
    });

    await waitFor(() => expect(global.fetch).toHaveBeenCalled());
    await waitFor(() =>
      expect(setStatus).toHaveBeenCalledWith("Verification complete.")
    );
  });

  it("surfaces API errors instead of throwing", async () => {
    const user = userEvent.setup();
    const setStatus = vi.fn();
    mockFetch({
      ok: false,
      status: 400,
      json: async () => ({ error: "Missing required claim: exp" }),
    });

    render(
      <TokenVerifier
        projectName="alpha"
        keys={[]}
        setStatus={setStatus}
      />
    );

    await act(async () => {
      const tokenField = screen.getByLabelText(/Token to Verify/i);
      await user.type(tokenField, makeToken({ sub: "user" }), {
        parseSpecialCharSequences: false,
      });
      await user.click(screen.getByLabelText(/Ignore Expiration/i));
      await user.click(screen.getByRole("button", { name: /Verify Token/i }));
    });

    await waitFor(() =>
      expect(setStatus).toHaveBeenCalledWith("Missing required claim: exp")
    );
  });

  it("sends verification fields per spec", async () => {
    const user = userEvent.setup();
    const setStatus = vi.fn();

    render(
      <TokenVerifier
        projectName="alpha"
        keys={[{ id: "key-1", name: "Primary" }]}
        setStatus={setStatus}
      />
    );

    await act(async () => {
      fireEvent.change(screen.getByLabelText(/Algorithm/i), {
        target: { value: "rs256" },
      });
      fireEvent.change(screen.getByLabelText(/Verification Key/i), {
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
      fireEvent.change(screen.getByLabelText(/Required Claims/i), {
        target: { value: "exp, nbf" },
      });
      fireEvent.change(screen.getByLabelText(/Leeway/i), {
        target: { value: "60" },
      });
      await user.click(screen.getByLabelText(/Try All Keys/i));
      await user.click(screen.getByLabelText(/Ignore Expiration/i));
      await user.click(screen.getByLabelText(/Provide Explanation/i));
      await user.type(
        screen.getByLabelText(/Token to Verify/i),
        makeToken({ sub: "user" }),
        { parseSpecialCharSequences: false }
      );
      await user.click(screen.getByRole("button", { name: /Verify Token/i }));
    });

    await waitFor(() => expect(global.fetch).toHaveBeenCalled());
    const verifyCall = global.fetch.mock.calls.find(
      ([url]) => url === "/api/jwt/verify"
    );
    expect(verifyCall).toBeTruthy();
    const body = JSON.parse(verifyCall[1].body);
    expect(body.project).toBe("alpha");
    expect(body.key_id).toBe("key-1");
    expect(body.alg).toBe("rs256");
    expect(body.try_all_keys).toBe(true);
    expect(body.ignore_exp).toBe(true);
    expect(body.leeway_secs).toBe(60);
    expect(body.iss).toBe("issuer");
    expect(body.sub).toBe("subject");
    expect(body.aud).toEqual(["api", "mobile"]);
    expect(body.require).toEqual(["exp", "nbf"]);
    expect(body.explain).toBe(true);
  });
});
