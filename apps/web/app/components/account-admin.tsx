"use client";

import { useEffect, useState } from "react";

import {
  getAccounts,
  setAccountRoles,
  type AccountSummary,
} from "../lib/csqd-api";
import { formatDate } from "../lib/public-audit";

const GRANTABLE_ROLES = ["sponsor", "reviewer", "operator"] as const;

/// Operator panel for granting roles. Role state used to require direct
/// SQL edits, which were provenance-invisible; this keeps grants inside
/// the product surface (and out of the database console).
export function AccountAdmin() {
  const [accounts, setAccounts] = useState<AccountSummary[] | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [pendingId, setPendingId] = useState<string | null>(null);

  const load = async () => {
    setAccounts(await getAccounts());
  };

  useEffect(() => {
    void load();
  }, []);

  const toggleRole = async (account: AccountSummary, role: string) => {
    const nextRoles = account.roles.includes(role)
      ? account.roles.filter((existing) => existing !== role)
      : [...account.roles, role];

    setPendingId(account.id);
    setError(null);

    try {
      await setAccountRoles(account.id, nextRoles);
      await load();
    } catch (updateError) {
      setError(
        updateError instanceof Error
          ? updateError.message
          : "Could not update roles.",
      );
    } finally {
      setPendingId(null);
    }
  };

  return (
    <section className="workspace-section">
      <div className="section-heading">
        <div>
          <p className="eyebrow">Identity and role state</p>
          <h2>Accounts</h2>
        </div>
      </div>
      {error ? <p className="form-error">{error}</p> : null}
      {accounts === null ? (
        <p className="muted-copy">Loading accounts…</p>
      ) : accounts.length === 0 ? (
        <div className="empty-state">
          <h2>No accounts yet</h2>
          <p>Accounts appear after first sign-in.</p>
        </div>
      ) : (
        <table className="console-table">
          <thead>
            <tr>
              <th>Account</th>
              <th>Joined</th>
              <th>Roles</th>
            </tr>
          </thead>
          <tbody>
            {accounts.map((account) => (
              <tr key={account.id}>
                <td>
                  <strong>{account.display_name}</strong>
                  <br />
                  {account.email}
                </td>
                <td>{formatDate(account.created_at)}</td>
                <td>
                  <div className="role-toggles">
                    {GRANTABLE_ROLES.map((role) => (
                      <label key={role}>
                        <input
                          checked={account.roles.includes(role)}
                          disabled={pendingId === account.id}
                          onChange={() => void toggleRole(account, role)}
                          type="checkbox"
                        />
                        {role}
                      </label>
                    ))}
                  </div>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
    </section>
  );
}
