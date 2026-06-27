export interface Account {
  Id: string;
  Remark: string;
  Username: string;
  LastUsed: string;
  GroupId: string;
  Tags: string[];
}

export interface Group {
  Id: string;
  Name: string;
  CreatedAt: string;
}

export interface UpdateInfo {
  version: string;
  notes: string | null;
  date: string | null;
}
