export interface Connection {
  id: string; name: string; database_type: 'postgresql'; host: string; port: number;
  database_name: string; username: string; has_password: boolean; read_only: boolean;
  ssl_mode: 'disable' | 'verify_full'; created_at: string; updated_at: string;
}
export type ConnectionInput = Omit<Connection, 'id' | 'has_password' | 'created_at' | 'updated_at'> & {password?: string};
export interface TableInfo {name: string; kind: string}
export interface ColumnInfo {name: string; type: string; nullable: boolean}
export interface QueryResult {
  columns: {name: string; type: string}[];
  rows: (string | number | boolean | null)[][];
  row_count: number; affected_rows: number; execution_time_ms: number; truncated: boolean;
}
